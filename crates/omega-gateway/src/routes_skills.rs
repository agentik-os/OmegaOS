//! Authenticated skill catalog, detail, safe edit, and rmux delegation.

use crate::protocol::{
    CreateSessionRequest, SkillAgentRequest, SkillAgentResponse, SkillDetail, SkillDetailResponse,
    SkillEntry, SkillUpdateRequest, SkillsResponse,
};
use crate::server::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use omega_core::skill_registry::{Skill, SkillRegistry};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path as FsPath, PathBuf};
use std::sync::{Mutex, OnceLock};

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 200;
const MAX_SKILL_CONTENT_BYTES: usize = 2 * 1024 * 1024;

type ApiError = (StatusCode, Json<serde_json::Value>);

fn api_error(status: StatusCode, message: impl Into<String>) -> ApiError {
    (status, Json(serde_json::json!({ "error": message.into() })))
}

fn write_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn safe_skill_path(root: &FsPath, skill: &Skill) -> anyhow::Result<PathBuf> {
    let root = root.canonicalize()?;
    let metadata = fs::symlink_metadata(&skill.path)?;
    anyhow::ensure!(
        !metadata.file_type().is_symlink(),
        "symlinked skill files are not editable"
    );
    let path = skill.path.canonicalize()?;
    anyhow::ensure!(
        path.starts_with(&root),
        "skill path escapes the owned skills root"
    );
    anyhow::ensure!(
        path.file_name().and_then(|name| name.to_str()) == Some("SKILL.md"),
        "invalid skill file"
    );
    Ok(path)
}

fn detail_from(root: &FsPath, skill: &Skill) -> anyhow::Result<SkillDetail> {
    let path = safe_skill_path(root, skill)?;
    let content = fs::read_to_string(path)?;
    Ok(SkillDetail {
        name: skill.name.clone(),
        description: skill.description.clone(),
        category: skill.category.label().to_string(),
        content,
        read_only: skill.read_only,
    })
}

fn atomic_replace(path: &FsPath, bytes: &[u8]) -> anyhow::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("skill file has no parent"))?;
    let temporary = parent.join(format!(".SKILL.md.{}.tmp", crate::util::random_hex(8)));
    let result = (|| -> anyhow::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        fs::set_permissions(&temporary, fs::metadata(path)?.permissions())?;
        fs::rename(&temporary, path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn update_at_root(root: &FsPath, name: &str, content: &str) -> anyhow::Result<SkillDetail> {
    anyhow::ensure!(
        !content.contains('\0'),
        "skill content must not contain a NUL byte"
    );
    anyhow::ensure!(
        content.len() <= MAX_SKILL_CONTENT_BYTES,
        "skill content exceeds the 2 MiB limit"
    );
    let _guard = write_lock()
        .lock()
        .map_err(|_| anyhow::anyhow!("skill write lock poisoned"))?;

    let registry = SkillRegistry::discover(root)?;
    let skill = registry
        .get(name)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("skill not found"))?;
    anyhow::ensure!(!skill.read_only, "skill is read-only");
    let path = safe_skill_path(root, &skill)?;
    let original = fs::read(&path)?;

    atomic_replace(&path, content.as_bytes())?;
    let refreshed = SkillRegistry::discover(root);
    match refreshed.and_then(|registry| {
        let skill = registry.get(name).ok_or_else(|| {
            anyhow::anyhow!("edited content changed or invalidated the skill identity")
        })?;
        detail_from(root, skill)
    }) {
        Ok(detail) => Ok(detail),
        Err(error) => {
            atomic_replace(&path, &original)?;
            Err(error)
        }
    }
}

pub async fn list(Query(params): Query<HashMap<String, String>>) -> Json<SkillsResponse> {
    let response = tokio::task::spawn_blocking(move || {
        let registry = match SkillRegistry::discover_default() {
            Ok(registry) => registry,
            Err(error) => {
                tracing::warn!("skill discovery failed: {error}");
                return SkillsResponse {
                    skills: vec![],
                    total: 0,
                };
            }
        };

        let all = registry.list();
        let total = all.len();
        let query = params
            .get("q")
            .map(|value| value.to_lowercase())
            .filter(|value| !value.is_empty());
        let category = params
            .get("category")
            .map(|value| value.to_lowercase())
            .filter(|value| !value.is_empty() && value != "all");
        let limit = params
            .get("limit")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(DEFAULT_LIMIT)
            .min(MAX_LIMIT);

        let skills = all
            .into_iter()
            .filter(|skill| {
                category
                    .as_ref()
                    .map(|value| skill.category.label().to_lowercase() == *value)
                    .unwrap_or(true)
            })
            .filter(|skill| match &query {
                None => true,
                Some(value) => {
                    skill.name.to_lowercase().contains(value)
                        || skill.description.to_lowercase().contains(value)
                }
            })
            .take(limit)
            .map(|skill| SkillEntry {
                name: skill.name.clone(),
                description: skill.description.clone(),
                category: skill.category.label().to_string(),
            })
            .collect();

        SkillsResponse { skills, total }
    })
    .await
    .unwrap_or(SkillsResponse {
        skills: vec![],
        total: 0,
    });
    Json(response)
}

pub async fn get(Path(name): Path<String>) -> Result<Json<SkillDetailResponse>, ApiError> {
    let detail = tokio::task::spawn_blocking(move || {
        let registry = SkillRegistry::discover_default()?;
        let skill = registry
            .get(&name)
            .ok_or_else(|| anyhow::anyhow!("skill not found"))?;
        detail_from(registry.skills_dir(), skill)
    })
    .await
    .map_err(|error| {
        api_error(
            StatusCode::BAD_GATEWAY,
            format!("skill task panicked: {error}"),
        )
    })?
    .map_err(|error| {
        let status = if error.to_string() == "skill not found" {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::BAD_REQUEST
        };
        api_error(status, error.to_string())
    })?;
    Ok(Json(SkillDetailResponse { skill: detail }))
}

pub async fn update(
    Path(name): Path<String>,
    Json(request): Json<SkillUpdateRequest>,
) -> Result<Json<SkillDetailResponse>, ApiError> {
    let detail = tokio::task::spawn_blocking(move || {
        let registry = SkillRegistry::discover_default()?;
        update_at_root(registry.skills_dir(), &name, &request.content)
    })
    .await
    .map_err(|error| {
        api_error(
            StatusCode::BAD_GATEWAY,
            format!("skill task panicked: {error}"),
        )
    })?
    .map_err(|error| {
        let message = error.to_string();
        let status = if message == "skill not found" {
            StatusCode::NOT_FOUND
        } else if message == "skill is read-only" {
            StatusCode::FORBIDDEN
        } else {
            StatusCode::BAD_REQUEST
        };
        api_error(status, message)
    })?;
    Ok(Json(SkillDetailResponse { skill: detail }))
}

pub async fn ask_agent(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(request): Json<SkillAgentRequest>,
) -> Result<Json<SkillAgentResponse>, ApiError> {
    let (directory, read_only) = tokio::task::spawn_blocking({
        let name = name.clone();
        move || -> anyhow::Result<(String, bool)> {
            let registry = SkillRegistry::discover_default()?;
            let skill = registry
                .get(&name)
                .ok_or_else(|| anyhow::anyhow!("skill not found"))?;
            let path = safe_skill_path(registry.skills_dir(), skill)?;
            Ok((
                path.parent()
                    .expect("validated skill parent")
                    .to_string_lossy()
                    .into_owned(),
                skill.read_only,
            ))
        }
    })
    .await
    .map_err(|error| {
        api_error(
            StatusCode::BAD_GATEWAY,
            format!("skill task panicked: {error}"),
        )
    })?
    .map_err(|error| {
        let status = if error.to_string() == "skill not found" {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::BAD_REQUEST
        };
        api_error(status, error.to_string())
    })?;
    if read_only {
        return Err(api_error(StatusCode::FORBIDDEN, "skill is read-only"));
    }

    let operator_request = request.prompt.unwrap_or_else(|| {
        "Review this skill and improve it where runtime evidence supports a change.".to_string()
    });
    let prompt = format!(
        "Modify the OmegaOS skill in ./SKILL.md. Preserve its identity, follow the repository AGENTS.md, verify the final file, and report exact evidence. Operator request: {operator_request}"
    );
    let created = crate::routes_sessions::create(
        State(state),
        Json(CreateSessionRequest {
            agent: "codex".to_string(),
            name: None,
            dir: Some(directory),
            prompt: Some(prompt),
        }),
    )
    .await?;
    Ok(Json(SkillAgentResponse {
        session: created.0.name,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skill(root: &FsPath, name: &str, read_only: bool) {
        let directory = root.join(name);
        fs::create_dir_all(&directory).unwrap();
        fs::write(
            directory.join("SKILL.md"),
            format!(
                "---\nname: {name}\ndescription: Original\nread_only: {read_only}\n---\n# Original\n"
            ),
        )
        .unwrap();
    }

    #[test]
    fn update_is_atomic_and_preserves_identity() {
        let root = tempfile::tempdir().unwrap();
        skill(root.path(), "alpha", false);
        let detail = update_at_root(
            root.path(),
            "alpha",
            "---\nname: alpha\ndescription: Updated\nread_only: false\n---\n# Updated\n",
        )
        .unwrap();
        assert_eq!(detail.description, "Updated");

        let error = update_at_root(
            root.path(),
            "alpha",
            "---\nname: renamed\ndescription: Bad\n---\n",
        )
        .unwrap_err();
        assert!(error.to_string().contains("identity"));
        assert!(fs::read_to_string(root.path().join("alpha/SKILL.md"))
            .unwrap()
            .contains("name: alpha"));
    }

    #[test]
    fn update_rejects_read_only_and_symlinked_files() {
        let root = tempfile::tempdir().unwrap();
        skill(root.path(), "locked", true);
        assert_eq!(
            update_at_root(root.path(), "locked", "changed")
                .unwrap_err()
                .to_string(),
            "skill is read-only"
        );

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let outside = root.path().join("outside.md");
            fs::write(&outside, "# Outside").unwrap();
            let directory = root.path().join("linked");
            fs::create_dir(&directory).unwrap();
            symlink(&outside, directory.join("SKILL.md")).unwrap();
            let fake = Skill {
                name: "linked".to_string(),
                description: String::new(),
                path: directory.join("SKILL.md"),
                triggers: vec![],
                phases: None,
                max_score: None,
                read_only: false,
                category: omega_core::skill_registry::SkillCategory::Custom,
                discovered_at: chrono::Utc::now(),
            };
            assert!(safe_skill_path(root.path(), &fake)
                .unwrap_err()
                .to_string()
                .contains("symlinked"));
        }
    }
}
