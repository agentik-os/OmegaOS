//! Pure deposit logic — the native Rust reimplementation of the Telegram
//! DEPOSIT bot's fan-out (`~/.omega/telegram-bot/inbox-bot.ts`), read in full
//! as the ground-truth spec for this module. No HTTP here; `routes_deposit.rs`
//! is the thin HTTP wrapper around [`deposit`].
//!
//! Storage layout under `deposit_home` (mirrors the real `~/.omega` layout):
//! `<deposit_home>/inbox/<timestamp>_<sanitized-name>`   (the original, always)
//! `<deposit_home>/deposit/<Box>/<timestamp>_<sanitized-name>`  (hard link, or
//!   a copy when the link fails, e.g. cross-filesystem)
//! `<deposit_home>/deposit/index.jsonl`  (one JSON line per deposit, held or not)
//! `<deposit_home>/deposit.toml`  (`DepositConfig`)

use serde::Deserialize;
use std::path::Path;

/// `boxes = [...]` / `fanout_secrets = true|false` in `<deposit_home>/deposit.toml`.
/// Same load idiom as `GatewayConfig::load`: missing or invalid file both
/// yield defaults (a corrupt file warns, never panics).
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct DepositConfig {
    pub boxes: Vec<String>,
    pub fanout_secrets: bool,
}

impl Default for DepositConfig {
    fn default() -> Self {
        Self {
            boxes: vec!["Home".into(), "AltReality".into(), "Omega".into(), "Box".into()],
            fanout_secrets: false,
        }
    }
}

impl DepositConfig {
    pub fn load(deposit_home: &Path) -> Self {
        let path = deposit_home.join("deposit.toml");
        match std::fs::read_to_string(&path) {
            Ok(text) => toml::from_str(&text).unwrap_or_else(|e| {
                tracing::warn!("invalid {}: {e}; using defaults", path.display());
                Self::default()
            }),
            Err(_) => Self::default(),
        }
    }
}

/// Outcome of one [`deposit`] call.
#[derive(Debug)]
pub struct DepositOutcome {
    /// The final timestamped filename written to the inbox.
    pub file: String,
    /// Boxes the deposit actually reached (empty when `held` is true).
    pub boxes: Vec<String>,
    /// True when the upload looked like a credential and was kept in the
    /// inbox only (see [`looks_secret`]).
    pub held: bool,
}

/// Sanitizes `original_name`, writes `bytes` into the inbox under a
/// timestamped filename, then (unless secret-looking and not forced)
/// hard-links (copy-fallback) that same file into each target box under
/// `<deposit_home>/deposit/`, and appends one line to
/// `<deposit_home>/deposit/index.jsonl`.
///
/// `box_tag`: `None` fans out to every configured box; `Some(name)` narrows
/// to that one box (case-insensitive match against the configured box
/// names). An unrecognized `box_tag` is rejected with `Err` BEFORE anything
/// is written to disk — not even the inbox original — matching the
/// path-traversal-guard discipline `valid_chat_id`/`valid_session_name`
/// already use elsewhere in this crate: validate first, touch the
/// filesystem second.
pub fn deposit(
    deposit_home: &Path,
    original_name: &str,
    bytes: &[u8],
    box_tag: Option<&str>,
    share: bool,
) -> anyhow::Result<DepositOutcome> {
    let config = DepositConfig::load(deposit_home);

    // Validate the box tag BEFORE touching the filesystem at all.
    let requested_box: Option<String> = match box_tag {
        Some(tag) => match config.boxes.iter().find(|b| b.eq_ignore_ascii_case(tag)) {
            Some(b) => Some(b.clone()),
            None => anyhow::bail!("unknown box: {tag}"),
        },
        None => None,
    };

    let sanitized = sanitize_filename(original_name);
    let filename = format!("{}_{}", timestamp(), sanitized);

    let inbox_dir = deposit_home.join("inbox");
    std::fs::create_dir_all(&inbox_dir)?;
    let inbox_path = inbox_dir.join(&filename);
    std::fs::write(&inbox_path, bytes)?;

    // A credential-looking file never silently fans out into a shared box —
    // unless the caller explicitly asked to share it, or the operator's own
    // config opts every deposit into fan-out.
    let held = looks_secret(&sanitized) && !share && !config.fanout_secrets;

    let boxes_dir = deposit_home.join("deposit");
    let mut reached: Vec<String> = Vec::new();
    if !held {
        let targets: Vec<String> = match requested_box {
            Some(b) => vec![b],
            None => config.boxes.clone(),
        };
        std::fs::create_dir_all(&boxes_dir)?;
        for b in &targets {
            let bdir = boxes_dir.join(b);
            std::fs::create_dir_all(&bdir)?;
            let dst = bdir.join(&filename);
            // Zero-extra-disk fan-out: hard-link, falling back to a copy when
            // the link fails (e.g. cross-filesystem) — mirrors inbox-bot.ts's
            // `place()` exactly.
            if std::fs::hard_link(&inbox_path, &dst).is_err() {
                std::fs::copy(&inbox_path, &dst)?;
            }
            reached.push(b.clone());
        }
    }

    // One index line per deposit, held or not, so the log is a complete
    // record of everything that arrived.
    std::fs::create_dir_all(&boxes_dir)?;
    let index_path = boxes_dir.join("index.jsonl");
    let line = serde_json::json!({
        "ts": timestamp(),
        "file": filename,
        "boxes": reached,
        "held": held,
    });
    {
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new().create(true).append(true).open(&index_path)?;
        writeln!(f, "{line}")?;
    }

    Ok(DepositOutcome { file: filename, boxes: reached, held })
}

/// The exact secret-detection regex the real Telegram DEPOSIT bot uses
/// (`SECRETISH` in `inbox-bot.ts`), case-insensitive, matched against the
/// (already-sanitized) base filename only.
fn looks_secret(filename: &str) -> bool {
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| {
        regex::Regex::new(
            r"(?i)(\.(p8|pem|key|env|p12|jks|keystore|ppk|crt|pfx)$)|(^|[._-])(id_rsa|id_ed25519)|credential|secret|token|passwd|private[._-]?key",
        )
        .expect("static secret-detection regex is valid")
    });
    re.is_match(filename)
}

/// Strips anything outside `[A-Za-z0-9._-]` to `_` and caps the result at
/// 100 chars. Never empty — falls back to `"file"` when sanitization leaves
/// nothing. Because `/` (and every other separator) is replaced, the result
/// is always a single path component: joined onto a directory it can never
/// path-traverse out of it, even when `original_name` is `../../etc/passwd`-shaped.
fn sanitize_filename(name: &str) -> String {
    let mut sanitized: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-' { c } else { '_' })
        .collect();
    sanitized.truncate(100);
    if sanitized.is_empty() {
        sanitized = "file".to_string();
    }
    sanitized
}

/// `YYYY-MM-DD_HHMMSS`, matching the Telegram bot's `stamp()` shape exactly.
fn timestamp() -> String {
    chrono::Utc::now().format("%Y-%m-%d_%H%M%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    fn ino(path: &Path) -> u64 {
        use std::os::unix::fs::MetadataExt;
        std::fs::metadata(path).unwrap().ino()
    }

    #[test]
    fn no_box_tag_fans_out_to_all_four_default_boxes_as_hard_links() {
        let dir = tempfile::tempdir().unwrap();
        let outcome = deposit(dir.path(), "notes.txt", b"hello", None, false).unwrap();

        assert!(!outcome.held);
        let mut boxes = outcome.boxes.clone();
        boxes.sort();
        assert_eq!(boxes, vec!["AltReality", "Box", "Home", "Omega"]);

        let inbox_path = dir.path().join("inbox").join(&outcome.file);
        assert!(inbox_path.exists());

        #[cfg(unix)]
        {
            let inbox_ino = ino(&inbox_path);
            for b in ["Home", "AltReality", "Omega", "Box"] {
                let box_path = dir.path().join("deposit").join(b).join(&outcome.file);
                assert!(box_path.exists());
                assert_eq!(ino(&box_path), inbox_ino, "{b} copy is not a hard link");
            }
        }
    }

    #[test]
    fn specific_box_tag_lands_only_in_that_box() {
        let dir = tempfile::tempdir().unwrap();
        let outcome = deposit(dir.path(), "notes.txt", b"hello", Some("omega"), false).unwrap();

        assert_eq!(outcome.boxes, vec!["Omega".to_string()]);
        assert!(dir.path().join("deposit").join("Omega").join(&outcome.file).exists());
        for b in ["Home", "AltReality", "Box"] {
            assert!(!dir.path().join("deposit").join(b).join(&outcome.file).exists());
        }
    }

    #[test]
    fn unrecognized_box_tag_errors_before_writing_anything() {
        let dir = tempfile::tempdir().unwrap();
        let err = deposit(dir.path(), "notes.txt", b"hello", Some("Nowhere"), false).unwrap_err();
        assert!(err.to_string().contains("Nowhere"));
        assert!(!dir.path().join("inbox").exists(), "inbox must stay untouched on validation failure");
        assert!(!dir.path().join("deposit").exists(), "no box dir may be created on validation failure");
    }

    #[test]
    fn secret_looking_file_is_held_by_default() {
        let dir = tempfile::tempdir().unwrap();
        let outcome = deposit(dir.path(), "id_rsa", b"private", None, false).unwrap();

        assert!(outcome.held);
        assert!(outcome.boxes.is_empty());
        assert!(dir.path().join("inbox").join(&outcome.file).exists());
        for b in ["Home", "AltReality", "Omega", "Box"] {
            let box_dir = dir.path().join("deposit").join(b);
            let has_files = box_dir.exists() && std::fs::read_dir(&box_dir).unwrap().next().is_some();
            assert!(!has_files, "{b} must not receive a held file");
        }
    }

    #[test]
    fn secret_looking_file_shares_when_share_is_true() {
        let dir = tempfile::tempdir().unwrap();
        let outcome = deposit(dir.path(), "service.pem", b"cert", None, true).unwrap();

        assert!(!outcome.held);
        assert_eq!(outcome.boxes.len(), 4);
        for b in ["Home", "AltReality", "Omega", "Box"] {
            assert!(dir.path().join("deposit").join(b).join(&outcome.file).exists());
        }
    }

    #[test]
    fn secret_looking_file_shares_when_fanout_secrets_configured() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("deposit.toml"), "fanout_secrets = true\n").unwrap();

        let outcome = deposit(dir.path(), ".env", b"SECRET=1", None, false).unwrap();

        assert!(!outcome.held);
        assert_eq!(outcome.boxes.len(), 4);
    }

    #[test]
    fn filename_sanitization_prevents_path_traversal() {
        let dir = tempfile::tempdir().unwrap();
        let outcome = deposit(dir.path(), "../../etc/passwd", b"x", None, false).unwrap();

        // The written file must live directly under inbox/, never having
        // escaped it via a slash surviving sanitization.
        let inbox_dir = dir.path().join("inbox");
        let entries: Vec<_> = std::fs::read_dir(&inbox_dir).unwrap().collect();
        assert_eq!(entries.len(), 1);
        assert!(!outcome.file.contains('/'));
        assert!(inbox_dir.join(&outcome.file).exists());
        // Nothing was written outside the tempdir root.
        assert!(!dir.path().join("etc").exists());
    }

    #[test]
    fn sanitize_filename_never_empty() {
        assert_eq!(sanitize_filename(""), "file");
        // Disallowed characters map to '_', which is itself a legal filename
        // character — that's a valid (if odd) non-empty result, not a case
        // for the "file" fallback.
        assert_eq!(sanitize_filename("///"), "___");
    }

    #[test]
    fn looks_secret_matches_the_telegram_bot_regex() {
        for name in ["id_rsa", "id_ed25519", "service.pem", ".env", "my.key", "app.p12", "a_secret_note.txt", "TOKEN.txt", "private-key.bin"] {
            assert!(looks_secret(name), "{name} should look secret");
        }
        for name in ["notes.txt", "photo.jpg", "report.pdf"] {
            assert!(!looks_secret(name), "{name} should not look secret");
        }
    }

    mod deposit_config {
        use super::*;

        #[test]
        fn missing_file_yields_defaults() {
            let dir = tempfile::tempdir().unwrap();
            let cfg = DepositConfig::load(dir.path());
            assert_eq!(cfg.boxes, vec!["Home", "AltReality", "Omega", "Box"]);
            assert!(!cfg.fanout_secrets);
        }

        #[test]
        fn file_overrides_fields() {
            let dir = tempfile::tempdir().unwrap();
            std::fs::write(
                dir.path().join("deposit.toml"),
                "boxes = [\"Only\"]\nfanout_secrets = true\n",
            )
            .unwrap();
            let cfg = DepositConfig::load(dir.path());
            assert_eq!(cfg.boxes, vec!["Only"]);
            assert!(cfg.fanout_secrets);
        }

        #[test]
        fn invalid_file_yields_defaults() {
            let dir = tempfile::tempdir().unwrap();
            std::fs::write(dir.path().join("deposit.toml"), "not valid toml {{{").unwrap();
            let cfg = DepositConfig::load(dir.path());
            assert_eq!(cfg.boxes, vec!["Home", "AltReality", "Omega", "Box"]);
            assert!(!cfg.fanout_secrets);
        }
    }
}
