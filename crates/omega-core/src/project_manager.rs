//! Project CRUD — create, register, discover, persist projects.

use anyhow::{Context, Result};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ManagedProject {
    pub name: String,
    pub path: PathBuf,
    pub telegram_topic_id: Option<i64>,
    pub oracle_session: Option<String>,
    pub git_email: Option<String>,
    pub created_at: String,
    /// Per-project Telegram visibility toggle. `None` or `Some(true)` = enabled
    /// (the project gets a forum topic on `/sync` and shows normally in the Atlas
    /// bot); `Some(false)` = disabled (sync skips/removes its topic, the bot marks
    /// it 🔕 but keeps it listed). Default ON preserves existing behavior.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub telegram: Option<bool>,
    /// Thematic category used to group the Projects tab. An explicit value wins;
    /// when unset it is derived per-machine from the project's own location. See
    /// `display_category`. Kept out of the serialized form when `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

/// Versioned fields carried alongside the legacy `ManagedProject` view.
///
/// `ManagedProject` is constructed as a public struct by downstream crates, so
/// adding fields to it would be a source-breaking change. The registry therefore
/// owns this forward-compatible fiche and flattens it into each project JSON
/// object. Unknown keys are retained in `extra`, which makes a load/mutate/save
/// cycle lossless for newer or third-party producers.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ProjectFiche {
    pub repo: Option<String>,
    pub slug: Option<String>,
    pub default_branch: Option<String>,
    pub deploy_target: Option<JsonValue>,
    pub credential_refs: Option<JsonValue>,
    pub key_locations: Option<JsonValue>,
    pub extra: JsonMap<String, JsonValue>,
    null_fields: BTreeSet<String>,
}

impl Serialize for ProjectFiche {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut fields = self.extra.clone();
        for name in &self.null_fields {
            fields.insert(name.clone(), JsonValue::Null);
        }
        if let Some(value) = &self.repo {
            fields.insert("repo".to_string(), JsonValue::String(value.clone()));
        }
        if let Some(value) = &self.slug {
            fields.insert("slug".to_string(), JsonValue::String(value.clone()));
        }
        if let Some(value) = &self.default_branch {
            fields.insert(
                "default_branch".to_string(),
                JsonValue::String(value.clone()),
            );
        }
        for (name, value) in [
            ("deploy_target", self.deploy_target.as_ref()),
            ("credential_refs", self.credential_refs.as_ref()),
            ("key_locations", self.key_locations.as_ref()),
        ] {
            if let Some(value) = value {
                fields.insert(name.to_string(), value.clone());
            }
        }
        JsonValue::Object(fields).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ProjectFiche {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut fields = JsonMap::<String, JsonValue>::deserialize(deserializer)?;
        let mut null_fields = BTreeSet::new();
        let mut take_string = |name: &str| -> std::result::Result<Option<String>, D::Error> {
            match fields.remove(name) {
                None => Ok(None),
                Some(JsonValue::Null) => {
                    null_fields.insert(name.to_string());
                    Ok(None)
                }
                Some(JsonValue::String(value)) => Ok(Some(value)),
                Some(_) => Err(serde::de::Error::custom(format!(
                    "project fiche field {name} must be a string or null"
                ))),
            }
        };
        let repo = take_string("repo")?;
        let slug = take_string("slug")?;
        let default_branch = take_string("default_branch")?;
        let mut take_value = |name: &str| -> Option<JsonValue> {
            match fields.remove(name) {
                Some(JsonValue::Null) => {
                    null_fields.insert(name.to_string());
                    None
                }
                value => value,
            }
        };
        let deploy_target = take_value("deploy_target");
        let credential_refs = take_value("credential_refs");
        let key_locations = take_value("key_locations");
        Ok(Self {
            repo,
            slug,
            default_branch,
            deploy_target,
            credential_refs,
            key_locations,
            extra: fields,
            null_fields,
        })
    }
}

impl ManagedProject {
    /// Whether this project participates in Telegram (topic sync + Atlas display).
    /// Enabled unless the toggle was explicitly set to `false`.
    pub fn telegram_enabled(&self) -> bool {
        self.telegram != Some(false)
    }

    /// Thematic category for the Projects-tab grouping. An explicit `category`
    /// (non-empty) wins; otherwise it is derived from the project's own location
    /// on THIS machine — the first path component under the user's configured
    /// projects root (`OmegaConfig::projects_dir`). A project that sits directly
    /// under the root (no category folder) or outside it is `"Other"`.
    ///
    /// Machine-agnostic by design: it mirrors whatever top-level folders the user
    /// organizes their projects into (customers/ side-business/ tools/ …, or any
    /// custom layout), never a hardcoded taxonomy. Pass the root from
    /// `OmegaConfig::projects_dir`.
    pub fn display_category(&self, projects_root: &Path) -> String {
        if let Some(c) = self.category.as_deref() {
            let c = c.trim();
            if !c.is_empty() {
                return c.to_string();
            }
        }
        // The first folder under the projects root, but only when the project
        // actually lives INSIDE a category folder (≥2 remaining components:
        // <category>/<project>[/…]). A bare child of the root has no category.
        if let Ok(rel) = self.path.strip_prefix(projects_root) {
            let mut comps = rel.components();
            if let Some(first) = comps.next() {
                if comps.next().is_some() {
                    return first.as_os_str().to_string_lossy().to_string();
                }
            }
        }
        "Other".to_string()
    }

    /// Sort key for the Projects-tab category sections: named categories
    /// alphabetically (case-insensitive), `"Other"` always last. Deliberately
    /// order-agnostic so it fits any user's folder taxonomy.
    pub fn category_rank(cat: &str) -> (u8, String) {
        if cat == "Other" {
            (1, String::new())
        } else {
            (0, cat.to_lowercase())
        }
    }
}

const PROJECT_REGISTRY_SCHEMA_VERSION: u32 = 1;
const PROJECT_KNOWN_FIELDS: &[&str] = &[
    "name",
    "path",
    "telegram_topic_id",
    "oracle_session",
    "git_email",
    "created_at",
    "telegram",
    "category",
];

#[derive(Debug, Clone, Default)]
pub struct ProjectRegistry {
    pub projects: Vec<ManagedProject>,
    /// True when this registry came from a file that exists but could not be
    /// read or parsed. Such a registry is empty-looking but MUST NOT be saved:
    /// writing it back would erase the user's real project list. Never
    /// serialized — it is a property of this load, not of the data.
    pub poisoned: bool,
    schema_version: u32,
    project_fiches: BTreeMap<String, ProjectFiche>,
    root_extra: JsonMap<String, JsonValue>,
    baseline: Option<RegistrySnapshot>,
}

#[derive(Debug, Clone, Default)]
struct RegistrySnapshot {
    projects: Vec<ManagedProject>,
    project_fiches: BTreeMap<String, ProjectFiche>,
    root_extra: JsonMap<String, JsonValue>,
}

impl RegistrySnapshot {
    fn capture(registry: &ProjectRegistry) -> Self {
        Self {
            projects: registry.projects.clone(),
            project_fiches: registry.project_fiches.clone(),
            root_extra: registry.root_extra.clone(),
        }
    }
}

fn project_key(project: &ManagedProject) -> String {
    format!(
        "{}\u{0}{}",
        project.path.to_string_lossy(),
        project.name.to_lowercase()
    )
}

fn registry_to_value(registry: &ProjectRegistry) -> Result<JsonValue> {
    let mut root = registry.root_extra.clone();
    root.insert(
        "schema_version".to_string(),
        JsonValue::from(registry.schema_version.max(PROJECT_REGISTRY_SCHEMA_VERSION)),
    );

    let mut projects = Vec::with_capacity(registry.projects.len());
    for project in &registry.projects {
        let mut object = match serde_json::to_value(project)? {
            JsonValue::Object(object) => object,
            _ => unreachable!("ManagedProject always serializes as an object"),
        };
        if let Some(fiche) = registry_fiche_for_index(registry, projects.len()) {
            let fiche_value = serde_json::to_value(fiche)?;
            if let JsonValue::Object(fields) = fiche_value {
                for (key, value) in fields {
                    anyhow::ensure!(
                        !PROJECT_KNOWN_FIELDS.contains(&key.as_str()),
                        "project fiche field {key:?} conflicts with a ManagedProject field"
                    );
                    object.insert(key, value);
                }
            }
        }
        projects.push(JsonValue::Object(object));
    }
    root.insert("projects".to_string(), JsonValue::Array(projects));
    Ok(JsonValue::Object(root))
}

impl Serialize for ProjectRegistry {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        registry_to_value(self)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ProjectRegistry {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = JsonValue::deserialize(deserializer)?;
        let mut root = match value {
            JsonValue::Object(root) => root,
            _ => {
                return Err(serde::de::Error::custom(
                    "project registry must be a JSON object",
                ))
            }
        };
        let schema_version = root
            .remove("schema_version")
            .and_then(|value| value.as_u64())
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(PROJECT_REGISTRY_SCHEMA_VERSION);
        let project_values = match root
            .remove("projects")
            .unwrap_or_else(|| JsonValue::Array(Vec::new()))
        {
            JsonValue::Array(projects) => projects,
            _ => return Err(serde::de::Error::custom("projects must be a JSON array")),
        };

        let mut projects = Vec::with_capacity(project_values.len());
        let mut project_fiches = BTreeMap::new();
        for value in project_values {
            let mut object = match value {
                JsonValue::Object(object) => object,
                _ => {
                    return Err(serde::de::Error::custom(
                        "each project must be a JSON object",
                    ))
                }
            };
            let project =
                serde_json::from_value::<ManagedProject>(JsonValue::Object(object.clone()))
                    .map_err(serde::de::Error::custom)?;
            for field in PROJECT_KNOWN_FIELDS {
                object.remove(*field);
            }
            if !object.is_empty() {
                let fiche = serde_json::from_value::<ProjectFiche>(JsonValue::Object(object))
                    .map_err(serde::de::Error::custom)?;
                project_fiches.insert(project_key(&project), fiche);
            }
            projects.push(project);
        }

        let mut registry = Self {
            projects,
            poisoned: false,
            schema_version,
            project_fiches,
            root_extra: root,
            baseline: None,
        };
        registry.baseline = Some(RegistrySnapshot::capture(&registry));
        Ok(registry)
    }
}

const REGISTRY_LOCK_WAIT: Duration = Duration::from_secs(5);
const REGISTRY_LOCK_STALE: Duration = Duration::from_secs(120);
static REGISTRY_TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct RegistryDirLock {
    path: PathBuf,
}

impl RegistryDirLock {
    fn acquire(registry_path: &Path) -> Result<Self> {
        let mut lock_name = registry_path.as_os_str().to_os_string();
        lock_name.push(".lock");
        let path = PathBuf::from(lock_name);
        let started = Instant::now();
        loop {
            match std::fs::create_dir(&path) {
                Ok(()) => {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))?;
                    }
                    return Ok(Self { path });
                }
                Err(error) if error.kind() == ErrorKind::AlreadyExists => {
                    let stale = std::fs::metadata(&path)
                        .and_then(|metadata| metadata.modified())
                        .ok()
                        .and_then(|modified| SystemTime::now().duration_since(modified).ok())
                        .is_some_and(|age| age >= REGISTRY_LOCK_STALE);
                    if stale && std::fs::remove_dir(&path).is_ok() {
                        continue;
                    }
                    anyhow::ensure!(
                        started.elapsed() < REGISTRY_LOCK_WAIT,
                        "timed out waiting for project registry lock {}",
                        path.display()
                    );
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(error) => {
                    return Err(error)
                        .with_context(|| format!("creating registry lock {}", path.display()));
                }
            }
        }
    }
}

impl Drop for RegistryDirLock {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir(&self.path) {
            eprintln!(
                "omega: could not release project registry lock {}: {error}",
                self.path.display()
            );
        }
    }
}

fn unique_temp_path(path: &Path) -> PathBuf {
    let sequence = REGISTRY_TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let mut name = path.as_os_str().to_os_string();
    name.push(format!(".tmp.{}.{}", std::process::id(), sequence));
    PathBuf::from(name)
}

fn write_registry_atomic(registry: &ProjectRegistry, path: &Path) -> Result<()> {
    let parent = path
        .parent()
        .with_context(|| format!("registry path has no parent: {}", path.display()))?;
    std::fs::create_dir_all(parent)?;
    let mut json = serde_json::to_vec_pretty(registry)?;
    json.push(b'\n');
    let tmp = unique_temp_path(path);

    let result = (|| -> Result<()> {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options
            .open(&tmp)
            .with_context(|| format!("creating temporary registry {}", tmp.display()))?;
        file.write_all(&json)
            .with_context(|| format!("writing temporary registry {}", tmp.display()))?;
        file.sync_all()
            .with_context(|| format!("fsync temporary registry {}", tmp.display()))?;
        drop(file);
        std::fs::rename(&tmp, path)
            .with_context(|| format!("atomically replacing registry {}", path.display()))?;
        #[cfg(unix)]
        File::open(parent)
            .and_then(|directory| directory.sync_all())
            .with_context(|| format!("fsync registry directory {}", parent.display()))?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

fn assign_unique_project_matches(
    baseline: &[ManagedProject],
    candidates: &[ManagedProject],
    matches: &mut [Option<usize>],
    used: &mut BTreeSet<usize>,
    predicate: impl Fn(usize, &ManagedProject, &ManagedProject) -> bool,
) {
    for (baseline_index, baseline_project) in baseline.iter().enumerate() {
        if matches[baseline_index].is_some() {
            continue;
        }
        let candidate_indices: Vec<_> = candidates
            .iter()
            .enumerate()
            .filter(|(candidate_index, candidate)| {
                !used.contains(candidate_index)
                    && predicate(baseline_index, baseline_project, candidate)
            })
            .map(|(candidate_index, _)| candidate_index)
            .collect();
        if let [candidate_index] = candidate_indices.as_slice() {
            matches[baseline_index] = Some(*candidate_index);
            used.insert(*candidate_index);
        }
    }
}

/// Match the same logical project across stale registry snapshots.
///
/// Name and path are editable user data, not durable identities. `created_at`
/// is the strongest legacy identity available, followed by the old composite
/// key, then one unchanged half of that key. The final positional fallback is
/// deliberately limited to equal-length snapshots, which covers an in-place
/// rename/move without mistaking a shifted item for a deletion plus addition.
fn project_correspondences(
    baseline: &[ManagedProject],
    candidates: &[ManagedProject],
) -> Vec<Option<usize>> {
    let mut matches = vec![None; baseline.len()];
    let mut used = BTreeSet::new();

    assign_unique_project_matches(
        baseline,
        candidates,
        &mut matches,
        &mut used,
        |baseline_index, project, candidate| {
            !project.created_at.is_empty()
                && !baseline.iter().enumerate().any(|(index, other)| {
                    index != baseline_index && other.created_at == project.created_at
                })
                && candidate.created_at == project.created_at
        },
    );
    assign_unique_project_matches(
        baseline,
        candidates,
        &mut matches,
        &mut used,
        |_, project, candidate| project_key(project) == project_key(candidate),
    );
    assign_unique_project_matches(
        baseline,
        candidates,
        &mut matches,
        &mut used,
        |_, project, candidate| project.path == candidate.path,
    );
    assign_unique_project_matches(
        baseline,
        candidates,
        &mut matches,
        &mut used,
        |_, project, candidate| project.name.eq_ignore_ascii_case(&candidate.name),
    );

    if baseline.len() == candidates.len() {
        for (index, matched) in matches.iter_mut().enumerate() {
            if matched.is_none() && !used.contains(&index) {
                *matched = Some(index);
                used.insert(index);
            }
        }
    }
    matches
}

fn baseline_key_for_current_index(
    registry: &ProjectRegistry,
    current_index: usize,
) -> Option<String> {
    let baseline = registry.baseline.as_ref()?;
    let matches = project_correspondences(&baseline.projects, &registry.projects);
    let baseline_index = matches
        .iter()
        .position(|candidate| *candidate == Some(current_index))?;
    Some(project_key(&baseline.projects[baseline_index]))
}

fn registry_fiche_for_index(
    registry: &ProjectRegistry,
    project_index: usize,
) -> Option<&ProjectFiche> {
    let project = registry.projects.get(project_index)?;
    let current_key = project_key(project);
    registry.project_fiches.get(&current_key).or_else(|| {
        baseline_key_for_current_index(registry, project_index)
            .and_then(|baseline_key| registry.project_fiches.get(&baseline_key))
    })
}

fn merge_json_map(
    baseline: &JsonMap<String, JsonValue>,
    desired: &JsonMap<String, JsonValue>,
    current: &JsonMap<String, JsonValue>,
) -> JsonMap<String, JsonValue> {
    let mut merged = current.clone();
    let keys: BTreeSet<_> = baseline.keys().chain(desired.keys()).cloned().collect();
    for key in keys {
        if desired.get(&key) == baseline.get(&key) {
            continue;
        }
        match desired.get(&key) {
            Some(value) => {
                merged.insert(key, value.clone());
            }
            None => {
                merged.remove(&key);
            }
        }
    }
    merged
}

fn merge_managed_project(
    baseline: &ManagedProject,
    desired: &ManagedProject,
    current: &ManagedProject,
) -> ManagedProject {
    let mut merged = current.clone();
    macro_rules! apply_local_change {
        ($field:ident) => {
            if desired.$field != baseline.$field {
                merged.$field = desired.$field.clone();
            }
        };
    }
    apply_local_change!(name);
    apply_local_change!(path);
    apply_local_change!(telegram_topic_id);
    apply_local_change!(oracle_session);
    apply_local_change!(git_email);
    apply_local_change!(created_at);
    apply_local_change!(telegram);
    apply_local_change!(category);
    merged
}

fn merge_project_fiche(
    baseline: Option<&ProjectFiche>,
    desired: &ProjectFiche,
    current: Option<&ProjectFiche>,
) -> ProjectFiche {
    let empty = ProjectFiche::default();
    let baseline = baseline.unwrap_or(&empty);
    let mut merged = current.cloned().unwrap_or_default();
    macro_rules! apply_local_change {
        ($field:ident) => {
            if desired.$field != baseline.$field {
                merged.$field = desired.$field.clone();
            }
        };
    }
    apply_local_change!(repo);
    apply_local_change!(slug);
    apply_local_change!(default_branch);
    apply_local_change!(deploy_target);
    apply_local_change!(credential_refs);
    apply_local_change!(key_locations);

    let extra_keys: BTreeSet<_> = baseline
        .extra
        .keys()
        .chain(desired.extra.keys())
        .cloned()
        .collect();
    for key in extra_keys {
        if desired.extra.get(&key) == baseline.extra.get(&key) {
            continue;
        }
        match desired.extra.get(&key) {
            Some(value) => {
                merged.extra.insert(key, value.clone());
            }
            None => {
                merged.extra.remove(&key);
            }
        }
    }

    let null_keys: BTreeSet<_> = baseline
        .null_fields
        .iter()
        .chain(desired.null_fields.iter())
        .cloned()
        .collect();
    for key in null_keys {
        let baseline_has = baseline.null_fields.contains(&key);
        let desired_has = desired.null_fields.contains(&key);
        if baseline_has == desired_has {
            continue;
        }
        if desired_has {
            merged.null_fields.insert(key);
        } else {
            merged.null_fields.remove(&key);
        }
    }
    merged
}

fn merge_registry_change(desired: &ProjectRegistry, latest: &ProjectRegistry) -> ProjectRegistry {
    let Some(baseline) = desired.baseline.as_ref() else {
        let mut authoritative = desired.clone();
        for project in &authoritative.projects {
            let key = project_key(project);
            if let std::collections::btree_map::Entry::Vacant(entry) =
                authoritative.project_fiches.entry(key.clone())
            {
                if let Some(fiche) = latest.project_fiches.get(&key) {
                    entry.insert(fiche.clone());
                }
            }
        }
        authoritative.root_extra = latest.root_extra.clone();
        authoritative.schema_version = desired
            .schema_version
            .max(latest.schema_version)
            .max(PROJECT_REGISTRY_SCHEMA_VERSION);
        authoritative.baseline = None;
        return authoritative;
    };

    let mut merged = latest.clone();
    let baseline_to_desired = project_correspondences(&baseline.projects, &desired.projects);
    let baseline_to_latest = project_correspondences(&baseline.projects, &latest.projects);
    let mut consumed_desired = BTreeSet::new();
    let mut consumed_latest = BTreeSet::new();
    let mut removed_latest = BTreeSet::new();
    let mut fiche_removals = BTreeSet::new();
    let mut fiche_insertions = Vec::new();

    for (baseline_index, baseline_project) in baseline.projects.iter().enumerate() {
        let desired_index = baseline_to_desired[baseline_index];
        let latest_index = baseline_to_latest[baseline_index];
        if let Some(index) = desired_index {
            consumed_desired.insert(index);
        }
        if let Some(index) = latest_index {
            consumed_latest.insert(index);
        }

        let baseline_key = project_key(baseline_project);
        let baseline_fiche = baseline.project_fiches.get(&baseline_key);
        let Some(desired_index) = desired_index else {
            if let Some(latest_index) = latest_index {
                removed_latest.insert(latest_index);
                fiche_removals.insert(project_key(&latest.projects[latest_index]));
            }
            fiche_removals.insert(baseline_key);
            continue;
        };

        let desired_project = &desired.projects[desired_index];
        let desired_key = project_key(desired_project);
        let desired_fiche = desired
            .project_fiches
            .get(&desired_key)
            .or_else(|| desired.project_fiches.get(&baseline_key));

        let (merged_project, latest_key, latest_fiche) = match latest_index {
            Some(index) => {
                let latest_project = &latest.projects[index];
                let latest_key = project_key(latest_project);
                let latest_fiche = latest
                    .project_fiches
                    .get(&latest_key)
                    .or_else(|| latest.project_fiches.get(&baseline_key));
                (
                    merge_managed_project(baseline_project, desired_project, latest_project),
                    Some(latest_key),
                    latest_fiche,
                )
            }
            None => (desired_project.clone(), None, None),
        };
        let merged_key = project_key(&merged_project);
        if let Some(index) = latest_index {
            merged.projects[index] = merged_project;
        } else {
            merged.projects.push(merged_project);
        }

        fiche_removals.insert(baseline_key);
        fiche_removals.insert(desired_key);
        if let Some(key) = latest_key {
            fiche_removals.insert(key);
        }
        let merged_fiche = if desired_fiche != baseline_fiche {
            desired_fiche.map(|fiche| merge_project_fiche(baseline_fiche, fiche, latest_fiche))
        } else {
            latest_fiche.cloned()
        };
        if let Some(fiche) = merged_fiche {
            fiche_insertions.push((merged_key, fiche));
        }
    }

    merged.projects = merged
        .projects
        .into_iter()
        .enumerate()
        .filter_map(|(index, project)| (!removed_latest.contains(&index)).then_some(project))
        .collect();

    for (desired_index, desired_project) in desired.projects.iter().enumerate() {
        if consumed_desired.contains(&desired_index) {
            continue;
        }
        let desired_key = project_key(desired_project);
        let existing_index = latest
            .projects
            .iter()
            .enumerate()
            .find(|(index, candidate)| {
                !consumed_latest.contains(index) && project_key(candidate) == desired_key
            })
            .map(|(index, _)| index);
        let latest_fiche = existing_index.and_then(|index| {
            let key = project_key(&latest.projects[index]);
            latest.project_fiches.get(&key)
        });
        if let Some(existing) = merged
            .projects
            .iter_mut()
            .find(|candidate| project_key(candidate) == desired_key)
        {
            *existing = desired_project.clone();
        } else {
            merged.projects.push(desired_project.clone());
        }
        fiche_removals.insert(desired_key.clone());
        if let Some(fiche) = desired.project_fiches.get(&desired_key) {
            fiche_insertions.push((desired_key, merge_project_fiche(None, fiche, latest_fiche)));
        }
    }

    for key in fiche_removals {
        merged.project_fiches.remove(&key);
    }
    for (key, fiche) in fiche_insertions {
        merged.project_fiches.insert(key, fiche);
    }

    merged.schema_version = desired
        .schema_version
        .max(latest.schema_version)
        .max(PROJECT_REGISTRY_SCHEMA_VERSION);
    merged.root_extra = merge_json_map(
        &baseline.root_extra,
        &desired.root_extra,
        &latest.root_extra,
    );
    merged.poisoned = false;
    merged.baseline = None;
    merged
}

impl ProjectRegistry {
    pub fn registry_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".omega")
            .join("projects.json")
    }

    pub fn load() -> Self {
        Self::load_from(&Self::registry_path())
    }

    /// Load the registry, treating "file is absent" and "file is unreadable"
    /// as DIFFERENT things.
    ///
    /// This used to be `.ok().and_then(parse).unwrap_or_default()`, which
    /// collapsed both into an empty registry. Callers then add one project and
    /// `save()` — so a single transient read failure silently replaced every
    /// project the user had with one entry. That is exactly what happened on
    /// 2026-07-29: a registry of 25 projects became 1.
    ///
    /// Absent → empty is correct (first run). Present-but-unreadable → we
    /// preserve the file, shout on stderr, and return a POISONED registry that
    /// `save()` refuses to persist, so the damage cannot be written back.
    pub fn load_from(path: &Path) -> Self {
        if !path.exists() {
            return Self::default();
        }
        match std::fs::read_to_string(path) {
            Ok(content) => match serde_json::from_str::<Self>(&content) {
                Ok(mut reg) => {
                    reg.poisoned = false;
                    reg
                }
                Err(e) => {
                    let backup = path.with_extension("json.unreadable");
                    let _ = std::fs::copy(path, &backup);
                    eprintln!(
                        "omega: projects.json could not be parsed ({e}). Your project list is \
                         NOT lost — the file is preserved at {}. Refusing to overwrite it.",
                        backup.display()
                    );
                    Self {
                        poisoned: true,
                        ..Self::default()
                    }
                }
            },
            Err(e) => {
                eprintln!(
                    "omega: projects.json could not be read ({e}). Refusing to overwrite it."
                );
                Self {
                    poisoned: true,
                    ..Self::default()
                }
            }
        }
    }

    pub fn save(&self) -> Result<()> {
        self.save_to(&Self::registry_path())
    }

    pub fn save_to(&self, path: &Path) -> Result<()> {
        // A registry we failed to read is never written back. Persisting it
        // would turn a transient read error into permanent data loss.
        anyhow::ensure!(
            !self.poisoned,
            "refusing to save a projects registry that failed to load — \
             the existing file is preserved"
        );
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let _lock = RegistryDirLock::acquire(path)?;
        let latest = if path.exists() {
            let latest = Self::load_from(path);
            anyhow::ensure!(
                !latest.poisoned,
                "refusing to replace a project registry that became unreadable while waiting for its lock"
            );
            latest
        } else {
            Self::default()
        };
        let merged = merge_registry_change(self, &latest);
        write_registry_atomic(&merged, path)
    }

    /// Execute one complete read-modify-write transaction under the registry's
    /// cross-language directory lock. New mutators should use this API instead
    /// of a separate `load()` and `save()` pair.
    pub fn update_locked<T>(
        path: &Path,
        update: impl FnOnce(&mut ProjectRegistry) -> Result<T>,
    ) -> Result<T> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let _lock = RegistryDirLock::acquire(path)?;
        let mut registry = Self::load_from(path);
        anyhow::ensure!(
            !registry.poisoned,
            "refusing to update a project registry that could not be loaded"
        );
        let output = update(&mut registry)?;
        registry.poisoned = false;
        write_registry_atomic(&registry, path)?;
        Ok(output)
    }

    pub fn fiche(&self, name: &str) -> Option<&ProjectFiche> {
        let project_index = self
            .projects
            .iter()
            .position(|project| project.name.eq_ignore_ascii_case(name))?;
        registry_fiche_for_index(self, project_index)
    }

    pub fn fiche_mut(&mut self, name: &str) -> Option<&mut ProjectFiche> {
        let project_index = self
            .projects
            .iter()
            .position(|project| project.name.eq_ignore_ascii_case(name))?;
        let key = project_key(&self.projects[project_index]);
        if !self.project_fiches.contains_key(&key) {
            if let Some(baseline_key) = baseline_key_for_current_index(self, project_index) {
                if baseline_key != key {
                    if let Some(fiche) = self.project_fiches.remove(&baseline_key) {
                        self.project_fiches.insert(key.clone(), fiche);
                    }
                }
            }
        }
        Some(self.project_fiches.entry(key).or_default())
    }

    pub fn find(&self, name: &str) -> Option<&ManagedProject> {
        let lower = name.to_lowercase();
        self.projects
            .iter()
            .find(|p| p.name.to_lowercase() == lower)
    }

    pub fn find_by_path(&self, path: &Path) -> Option<&ManagedProject> {
        self.projects.iter().find(|p| p.path == path)
    }

    pub fn add(&mut self, project: ManagedProject) {
        if self.find_by_path(&project.path).is_none() {
            self.projects.push(project);
        }
    }

    pub fn remove(&mut self, name: &str) -> bool {
        let lower = name.to_lowercase();
        let removed_keys: Vec<_> = self
            .projects
            .iter()
            .filter(|project| project.name.to_lowercase() == lower)
            .map(project_key)
            .collect();
        let before = self.projects.len();
        self.projects.retain(|p| p.name.to_lowercase() != lower);
        for key in removed_keys {
            self.project_fiches.remove(&key);
        }
        self.projects.len() < before
    }

    /// Set a project's Telegram toggle (topic sync + Atlas visibility). Returns
    /// `true` if a project with that name existed and was updated.
    pub fn set_telegram(&mut self, name: &str, enabled: bool) -> bool {
        let lower = name.to_lowercase();
        if let Some(p) = self
            .projects
            .iter_mut()
            .find(|p| p.name.to_lowercase() == lower)
        {
            p.telegram = Some(enabled);
            true
        } else {
            false
        }
    }
}

const SCAFFOLD_DIRS: &[&str] = &["docs", "docs/FEATURES", ".planner", ".oracles", ".omega"];

const CLAUDE_MD_TEMPLATE: &str = r#"# {name}

## Project

- **Path:** {path}
- **Created:** {date}

## Development Rules

- Law 1: Code lies. Only runtime tells the truth.
- Law 2: Researcher, not sycophant. Challenge flawed premises.
- Every feature must be verified with live runtime evidence before merge.
"#;

const VISION_TEMPLATE: &str = r#"# Vision — {name}

> Define the emotional foundation and product identity here.

## Internal Compass

_One sentence that acts as a decision filter for everything._

## Soul Statement

_What is this product, at its core?_

## Design Principles

1.
2.
3.

## Personas

### Primary

- **Name:**
- **Role:**
- **Pain:**
- **Goal:**

## What This Is NOT

-
"#;

const PRD_TEMPLATE: &str = r#"# PRD — {name}

> Product Requirements Document

## Overview

_What problem does this solve and for whom?_

## Goals

1.
2.
3.

## Features

| ID | Feature | Priority | Status |
|----|---------|----------|--------|
| F-001 | | P0 | planned |

## Technical Constraints

-

## Success Metrics

-
"#;

/// Scaffold a project on disk and describe it — WITHOUT touching the registry.
///
/// Split out from `create_project` so the scaffolding can be tested without
/// writing the machine's real `~/.omega/projects.json`. The old test called
/// `create_project` directly and rewrote the operator's registry every time the
/// suite ran; on 2026-07-29 that replaced 25 projects with one.
pub fn scaffold_project(name: &str, location: &Path) -> Result<ManagedProject> {
    let project_path = location.join(name);
    std::fs::create_dir_all(&project_path)
        .with_context(|| format!("creating project dir at {}", project_path.display()))?;

    for dir in SCAFFOLD_DIRS {
        std::fs::create_dir_all(project_path.join(dir))?;
    }

    let date = chrono::Utc::now().to_rfc3339();

    let claude_md = CLAUDE_MD_TEMPLATE
        .replace("{name}", name)
        .replace("{path}", &project_path.to_string_lossy())
        .replace("{date}", &date);
    std::fs::write(project_path.join("CLAUDE.md"), claude_md)?;

    let vision = VISION_TEMPLATE.replace("{name}", name);
    std::fs::write(project_path.join("docs/VISION.md"), vision)?;

    let prd = PRD_TEMPLATE.replace("{name}", name);
    std::fs::write(project_path.join("docs/PRD.md"), prd)?;

    // git init if not already a repo
    if !project_path.join(".git").exists() {
        let _ = std::process::Command::new("git")
            .args(["init"])
            .current_dir(&project_path)
            .output();
    }

    let project = ManagedProject {
        name: name.to_string(),
        path: project_path,
        telegram_topic_id: None,
        oracle_session: Some(format!("oracle-{}", name)),
        git_email: None,
        created_at: date,
        telegram: None,
        category: None,
    };

    Ok(project)
}

/// Scaffold a project AND register it, so `/projects` sees it. This is the
/// production entry point; tests use `scaffold_project` instead.
pub fn create_project(name: &str, location: &Path) -> Result<ManagedProject> {
    let project = scaffold_project(name, location)?;
    ProjectRegistry::update_locked(&ProjectRegistry::registry_path(), |registry| {
        registry.remove(name);
        registry.projects.push(project.clone());
        Ok(())
    })?;
    Ok(project)
}

pub fn add_existing_project(path: &Path) -> Result<ManagedProject> {
    anyhow::ensure!(path.exists(), "path does not exist: {}", path.display());
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let project = ManagedProject {
        name: name.clone(),
        path: path.to_path_buf(),
        telegram_topic_id: None,
        oracle_session: Some(format!("oracle-{}", name)),
        git_email: None,
        created_at: chrono::Utc::now().to_rfc3339(),
        telegram: None,
        category: None,
    };

    // Persist to the registry so /projects sees it next time.
    ProjectRegistry::update_locked(&ProjectRegistry::registry_path(), |registry| {
        // De-dup: replace any existing entry with the same name OR path.
        let path_str = path.display().to_string();
        let removed_keys: Vec<_> = registry
            .projects
            .iter()
            .filter(|candidate| {
                candidate.name == name || candidate.path.display().to_string() == path_str
            })
            .map(project_key)
            .collect();
        registry.projects.retain(|candidate| {
            candidate.name != name && candidate.path.display().to_string() != path_str
        });
        for key in removed_keys {
            registry.project_fiches.remove(&key);
        }
        registry.projects.push(project.clone());
        Ok(())
    })?;

    Ok(project)
}

pub fn scan_directory(root: &Path) -> Vec<ManagedProject> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(root) else {
        return found;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if !p.is_dir() {
            continue;
        }
        let Some(name) = p.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if name.starts_with('.') || name == "node_modules" || name == "target" {
            continue;
        }
        let is_project = p.join(".git").exists()
            || p.join("package.json").exists()
            || p.join("Cargo.toml").exists()
            || p.join("pyproject.toml").exists()
            || p.join("go.mod").exists();
        if is_project {
            found.push(ManagedProject {
                name: name.to_string(),
                path: p,
                telegram_topic_id: None,
                oracle_session: None,
                git_email: None,
                created_at: chrono::Utc::now().to_rfc3339(),
                telegram: None,
                category: None,
            });
        }
    }
    found.sort_by(|a, b| a.name.cmp(&b.name));
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: this test used to call `create_project()`, which persists to the
    // REAL ~/.omega/projects.json. Running the suite therefore rewrote the
    // machine's own project registry — on 2026-07-29 it replaced 25 projects
    // with a single "TestApp". It now exercises the scaffolding and the
    // registry separately, and never touches the user's registry.
    #[test]
    fn scaffolding_creates_the_expected_layout() {
        let tmp = tempfile::tempdir().unwrap();
        let project = scaffold_project("TestApp", tmp.path()).unwrap();
        assert_eq!(project.name, "TestApp");
        assert!(project.path.join("CLAUDE.md").exists());
        assert!(project.path.join("docs/VISION.md").exists());
        assert!(project.path.join("docs/PRD.md").exists());
        assert!(project.path.join(".planner").exists());
    }

    #[test]
    fn registry_add_is_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        let project = scaffold_project("TestApp", tmp.path()).unwrap();

        let mut registry = ProjectRegistry::default();
        registry.add(project.clone());
        assert_eq!(registry.projects.len(), 1);
        assert!(registry.find("testapp").is_some());

        registry.add(project);
        assert_eq!(registry.projects.len(), 1, "duplicate add is a no-op");
    }

    /// The data-loss guard: a registry that failed to load must never be
    /// written back over the file it failed to read.
    #[test]
    fn an_unreadable_registry_is_never_saved_over() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("projects.json");
        std::fs::write(&path, "{ this is not json").unwrap();

        let reg = ProjectRegistry::load_from(&path);
        assert!(
            reg.poisoned,
            "an unparseable registry must be marked poisoned"
        );
        assert!(reg.projects.is_empty());
        assert!(
            reg.save_to(&path).is_err(),
            "saving a poisoned registry would erase the user's projects"
        );
        // The original bytes survive, and a copy is preserved for recovery.
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "{ this is not json"
        );
        assert!(path.with_extension("json.unreadable").exists());
    }

    /// …but a genuinely absent registry is just a first run.
    #[test]
    fn an_absent_registry_is_a_clean_start() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("projects.json");
        let reg = ProjectRegistry::load_from(&path);
        assert!(!reg.poisoned);
        assert!(
            reg.save_to(&path).is_ok(),
            "a first run must be able to save"
        );
    }

    /// A save must be atomic — a concurrent reader seeing a half-written file
    /// is what poisons a load and starts the whole failure chain.
    #[test]
    fn save_round_trips_and_leaves_no_temp_file() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("projects.json");
        let scaffold_dir = tempfile::tempdir().unwrap();
        let mut reg = ProjectRegistry::default();
        reg.add(scaffold_project("A", scaffold_dir.path()).unwrap());
        reg.save_to(&path).unwrap();

        let back = ProjectRegistry::load_from(&path);
        assert_eq!(back.projects.len(), 1);
        assert!(!back.poisoned);
        assert!(
            !path.with_extension("json.tmp").exists(),
            "temp file must be renamed away"
        );
    }

    #[test]
    fn telegram_toggle_roundtrip() {
        // Absent field → None → enabled (default ON, backward-compatible).
        let legacy = r#"{"projects":[{"name":"A","path":"/p/a","telegram_topic_id":null,"oracle_session":null,"git_email":null,"created_at":"x"}]}"#;
        let reg: ProjectRegistry = serde_json::from_str(legacy).unwrap();
        assert!(
            reg.find("A").unwrap().telegram_enabled(),
            "absent telegram must default to enabled"
        );

        // Set OFF, serialize, and confirm it persists as `false` across a round-trip.
        let mut reg2 = reg.clone();
        assert!(reg2.set_telegram("a", false));
        assert!(!reg2.find("A").unwrap().telegram_enabled());
        let json = serde_json::to_string(&reg2).unwrap();
        assert!(
            json.contains("\"telegram\":false"),
            "OFF must serialize: {json}"
        );
        let reg3: ProjectRegistry = serde_json::from_str(&json).unwrap();
        assert!(
            !reg3.find("A").unwrap().telegram_enabled(),
            "OFF must survive round-trip"
        );

        // Flip back ON → enabled again, and the field is no longer `false`.
        let mut reg4 = reg3.clone();
        assert!(reg4.set_telegram("a", true));
        assert!(reg4.find("A").unwrap().telegram_enabled());
        assert!(!serde_json::to_string(&reg4)
            .unwrap()
            .contains("\"telegram\":false"));
    }

    #[test]
    fn ecosystem_project_fiche_and_unknown_fields_round_trip() {
        let input = serde_json::json!({
            "schema_version": 7,
            "registry_owner": "external-controller",
            "projects": [{
                "name": "A",
                "path": "/p/a",
                "telegram_topic_id": null,
                "oracle_session": null,
                "git_email": null,
                "created_at": "x",
                "repo": "https://example.invalid/acme/a.git",
                "slug": "acme/a",
                "default_branch": "trunk",
                "deploy_target": {"provider": "example", "project": "a"},
                "credential_refs": ["vault://projects/a"],
                "key_locations": {"env": ".env.enc"},
                "future_field": {"nested": [1, 2, 3]},
                "future_null": null
            }]
        });
        let mut registry: ProjectRegistry = serde_json::from_value(input.clone()).unwrap();
        let fiche = registry.fiche("a").expect("fiche must be available");
        assert_eq!(fiche.slug.as_deref(), Some("acme/a"));
        assert_eq!(fiche.default_branch.as_deref(), Some("trunk"));
        assert!(registry.set_telegram("A", false));

        let output = serde_json::to_value(&registry).unwrap();
        assert_eq!(output["schema_version"], 7);
        assert_eq!(output["registry_owner"], input["registry_owner"]);
        for field in [
            "repo",
            "slug",
            "default_branch",
            "deploy_target",
            "credential_refs",
            "key_locations",
            "future_field",
            "future_null",
        ] {
            assert_eq!(output["projects"][0][field], input["projects"][0][field]);
        }
        assert_eq!(output["projects"][0]["telegram"], false);
    }

    #[test]
    fn ecosystem_concurrent_stale_saves_merge_without_data_loss() {
        use std::sync::{Arc, Barrier};

        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("projects.json");
        std::fs::write(
            &path,
            serde_json::to_vec_pretty(&serde_json::json!({
                "root_unknown": "keep",
                "projects": [{
                    "name": "A",
                    "path": "/p/a",
                    "telegram_topic_id": null,
                    "oracle_session": null,
                    "git_email": null,
                    "created_at": "x",
                    "repo": "https://example.invalid/a.git",
                    "future": 42
                }]
            }))
            .unwrap(),
        )
        .unwrap();

        let mut toggle = ProjectRegistry::load_from(&path);
        let mut addition = ProjectRegistry::load_from(&path);
        assert!(toggle.set_telegram("A", false));
        toggle.fiche_mut("A").unwrap().slug = Some("acme/a".into());
        addition
            .projects
            .iter_mut()
            .find(|project| project.name == "A")
            .unwrap()
            .telegram_topic_id = Some(99);
        addition.fiche_mut("A").unwrap().default_branch = Some("main".into());
        addition.add(proj("/p/b", None));
        addition.projects.last_mut().unwrap().name = "B".into();

        let barrier = Arc::new(Barrier::new(3));
        let toggle_path = path.clone();
        let toggle_barrier = barrier.clone();
        let toggle_thread = std::thread::spawn(move || {
            toggle_barrier.wait();
            toggle.save_to(&toggle_path)
        });
        let addition_path = path.clone();
        let addition_barrier = barrier.clone();
        let addition_thread = std::thread::spawn(move || {
            addition_barrier.wait();
            addition.save_to(&addition_path)
        });
        barrier.wait();
        toggle_thread.join().unwrap().unwrap();
        addition_thread.join().unwrap().unwrap();

        let output: JsonValue = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let registry = ProjectRegistry::load_from(&path);
        assert_eq!(registry.projects.len(), 2);
        assert!(!registry.find("A").unwrap().telegram_enabled());
        assert_eq!(registry.find("A").unwrap().telegram_topic_id, Some(99));
        assert!(registry.find("B").is_some());
        let fiche = registry.fiche("A").unwrap();
        assert_eq!(fiche.slug.as_deref(), Some("acme/a"));
        assert_eq!(fiche.default_branch.as_deref(), Some("main"));
        assert_eq!(output["root_unknown"], "keep");
        let a = output["projects"]
            .as_array()
            .unwrap()
            .iter()
            .find(|project| project["name"] == "A")
            .unwrap();
        assert_eq!(a["repo"], "https://example.invalid/a.git");
        assert_eq!(a["future"], 42);
        assert!(!tmp.path().join("projects.json.lock").exists());
        assert!(std::fs::read_dir(tmp.path()).unwrap().all(|entry| {
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with("projects.json.tmp.")
        }));
    }

    #[test]
    fn ecosystem_concurrent_rename_move_and_fiche_changes_preserve_identity() {
        use std::sync::{Arc, Barrier};

        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("projects.json");
        std::fs::write(
            &path,
            serde_json::to_vec_pretty(&serde_json::json!({
                "projects": [{
                    "name": "Alpha",
                    "path": "/projects/alpha",
                    "telegram_topic_id": null,
                    "oracle_session": null,
                    "git_email": null,
                    "created_at": "2026-08-11T12:00:00Z",
                    "repo": "https://example.invalid/alpha.git",
                    "future": {"owner": "external"}
                }]
            }))
            .unwrap(),
        )
        .unwrap();

        let mut renamed = ProjectRegistry::load_from(&path);
        let mut enriched = ProjectRegistry::load_from(&path);
        renamed.fiche_mut("Alpha").unwrap().slug = Some("acme/omega".into());
        let renamed_project = renamed
            .projects
            .iter_mut()
            .find(|candidate| candidate.name == "Alpha")
            .unwrap();
        renamed_project.name = "Omega".into();
        renamed_project.path = PathBuf::from("/projects/moved/omega");
        enriched
            .projects
            .iter_mut()
            .find(|candidate| candidate.name == "Alpha")
            .unwrap()
            .telegram_topic_id = Some(77);
        enriched.fiche_mut("Alpha").unwrap().default_branch = Some("main".into());

        let barrier = Arc::new(Barrier::new(3));
        let renamed_path = path.clone();
        let renamed_barrier = barrier.clone();
        let renamed_thread = std::thread::spawn(move || {
            renamed_barrier.wait();
            renamed.save_to(&renamed_path)
        });
        let enriched_path = path.clone();
        let enriched_barrier = barrier.clone();
        let enriched_thread = std::thread::spawn(move || {
            enriched_barrier.wait();
            enriched.save_to(&enriched_path)
        });
        barrier.wait();
        renamed_thread.join().unwrap().unwrap();
        enriched_thread.join().unwrap().unwrap();

        let registry = ProjectRegistry::load_from(&path);
        assert!(registry.find("Alpha").is_none());
        let project = registry
            .find("Omega")
            .expect("renamed project must survive");
        assert_eq!(project.path, PathBuf::from("/projects/moved/omega"));
        assert_eq!(project.telegram_topic_id, Some(77));
        let fiche = registry
            .fiche("Omega")
            .expect("fiche must follow rename and move");
        assert_eq!(
            fiche.repo.as_deref(),
            Some("https://example.invalid/alpha.git")
        );
        assert_eq!(fiche.slug.as_deref(), Some("acme/omega"));
        assert_eq!(fiche.default_branch.as_deref(), Some("main"));
        assert_eq!(
            fiche.extra["future"],
            serde_json::json!({"owner": "external"})
        );
    }

    #[test]
    fn ecosystem_concurrent_disjoint_root_metadata_changes_merge_per_key() {
        use std::sync::{Arc, Barrier};

        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("projects.json");
        std::fs::write(
            &path,
            serde_json::to_vec_pretty(&serde_json::json!({
                "controller": {"revision": 1},
                "presentation": {"theme": "dark"},
                "projects": []
            }))
            .unwrap(),
        )
        .unwrap();

        let mut controller = ProjectRegistry::load_from(&path);
        let mut presentation = ProjectRegistry::load_from(&path);
        controller
            .root_extra
            .insert("controller".into(), serde_json::json!({"revision": 2}));
        presentation
            .root_extra
            .insert("presentation".into(), serde_json::json!({"theme": "light"}));

        let barrier = Arc::new(Barrier::new(3));
        let controller_path = path.clone();
        let controller_barrier = barrier.clone();
        let controller_thread = std::thread::spawn(move || {
            controller_barrier.wait();
            controller.save_to(&controller_path)
        });
        let presentation_path = path.clone();
        let presentation_barrier = barrier.clone();
        let presentation_thread = std::thread::spawn(move || {
            presentation_barrier.wait();
            presentation.save_to(&presentation_path)
        });
        barrier.wait();
        controller_thread.join().unwrap().unwrap();
        presentation_thread.join().unwrap().unwrap();

        let output: JsonValue = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(output["controller"], serde_json::json!({"revision": 2}));
        assert_eq!(
            output["presentation"],
            serde_json::json!({"theme": "light"})
        );
    }

    #[test]
    fn scan_finds_projects() {
        let tmp = std::env::temp_dir().join("omega-scan-test");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(tmp.join("project-a/.git")).unwrap();
        std::fs::create_dir_all(tmp.join("project-b")).unwrap();
        std::fs::write(tmp.join("project-b/package.json"), "{}").unwrap();
        std::fs::create_dir_all(tmp.join("not-a-project")).unwrap();

        let found = scan_directory(&tmp);
        assert_eq!(found.len(), 2);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    fn proj(path: &str, category: Option<&str>) -> ManagedProject {
        ManagedProject {
            name: "x".into(),
            path: PathBuf::from(path),
            telegram_topic_id: None,
            oracle_session: None,
            git_email: None,
            created_at: "t".into(),
            telegram: None,
            category: category.map(|s| s.into()),
        }
    }

    #[test]
    fn category_is_derived_from_the_users_own_layout() {
        // Machine-agnostic: the category is the first folder under the user's
        // configured projects root — whatever it is named — NOT a fixed taxonomy.
        let root = Path::new("/home/alice/dev");
        assert_eq!(
            proj("/home/alice/dev/customers/acme", None).display_category(root),
            "customers"
        );
        assert_eq!(
            proj("/home/alice/dev/tools/mylib/crate", None).display_category(root),
            "tools"
        );
        // A different user with a different root and different folder names.
        let root2 = Path::new("/Users/bob/Code");
        assert_eq!(
            proj("/Users/bob/Code/Clients/foo", None).display_category(root2),
            "Clients"
        );
    }

    #[test]
    fn category_falls_back_to_other() {
        let root = Path::new("/home/alice/dev");
        // Directly under the root (no category folder) → Other.
        assert_eq!(
            proj("/home/alice/dev/loose-project", None).display_category(root),
            "Other"
        );
        // Outside the root entirely → Other.
        assert_eq!(
            proj("/somewhere/else/proj", None).display_category(root),
            "Other"
        );
    }

    #[test]
    fn explicit_category_overrides_derivation() {
        let root = Path::new("/home/alice/dev");
        // Even though the path would derive "side-business", the pin wins.
        assert_eq!(
            proj("/home/alice/dev/side-business/omega", Some("Framework")).display_category(root),
            "Framework"
        );
        // Blank/whitespace category is ignored → falls back to derivation.
        assert_eq!(
            proj("/home/alice/dev/tools/x", Some("  ")).display_category(root),
            "tools"
        );
    }

    #[test]
    fn category_rank_sorts_named_alpha_other_last() {
        let mut cats = vec!["Partners", "Other", "framework", "Nova"];
        cats.sort_by_key(|c| ManagedProject::category_rank(c));
        assert_eq!(cats, vec!["framework", "Nova", "Partners", "Other"]);
    }
}
