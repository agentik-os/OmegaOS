//! Telegram media intake: voice / document / photo download + processing.
//!
//! The Telegram bridge receives three non-text modalities. This module turns
//! each into something the existing text pipeline can consume:
//!
//! - **Voice** → download the OGG/Opus blob via getFile, transcribe it with
//!   OpenAI Whisper (`/v1/audio/transcriptions`) using the stored `codex`
//!   (OPENAI) API key. The transcript is then routed exactly like a typed
//!   message.
//! - **Photo** → download the largest JPEG via getFile, describe it with the
//!   Anthropic Messages vision API using the stored `claude` API key. The
//!   description (plus any caption) is routed like a typed message.
//! - **Document** → download via getFile. Text-like files are read inline and
//!   routed as text; binary/large files are saved to the local inbox and the
//!   real path + size are reported.
//!
//! HONEST DEGRADATION: when a modality's processing path needs a credential or
//! capability that isn't present, the download + plumbing still run and the
//! user gets a clear message naming EXACTLY what is missing — never a silent
//! fake transcription/description.

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

const API_BASE: &str = "https://api.telegram.org";

/// Cap on bytes we'll pull from Telegram for any single intake file. Telegram's
/// own bot-download limit is 20 MB; we mirror that so a huge upload can't blow
/// memory. (getFile only ever exposes files the bot can download anyway.)
pub const MAX_DOWNLOAD_BYTES: u64 = 20 * 1024 * 1024;

#[derive(Debug, Deserialize)]
struct GetFileResp {
    ok: bool,
    #[serde(default)]
    result: Option<TgFile>,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TgFile {
    #[serde(default)]
    file_path: Option<String>,
    #[serde(default)]
    file_size: Option<u64>,
}

/// Resolve a `file_id` to its server `file_path` via the Telegram getFile API,
/// then download the bytes from the file CDN. Returns the raw blob.
///
/// Errors carry a human-readable reason (already token-free) so the caller can
/// surface them. Enforces [`MAX_DOWNLOAD_BYTES`].
pub async fn download_file(
    client: &reqwest::Client,
    token: &str,
    file_id: &str,
) -> Result<Vec<u8>> {
    let get_url = format!("{}/bot{}/getFile", API_BASE, token);
    let resp = client
        .post(&get_url)
        .json(&serde_json::json!({ "file_id": file_id }))
        .send()
        .await
        .context("getFile request failed")?;

    let parsed: GetFileResp = resp.json().await.context("getFile decode failed")?;
    if !parsed.ok {
        return Err(anyhow!(
            "getFile returned not-ok{}",
            parsed
                .description
                .map(|d| format!(": {}", d))
                .unwrap_or_default()
        ));
    }
    let file = parsed.result.ok_or_else(|| anyhow!("getFile: empty result"))?;
    if let Some(size) = file.file_size {
        if size > MAX_DOWNLOAD_BYTES {
            return Err(anyhow!(
                "file is {} bytes — over the {} byte intake cap",
                size,
                MAX_DOWNLOAD_BYTES
            ));
        }
    }
    let file_path = file
        .file_path
        .ok_or_else(|| anyhow!("getFile: no file_path (file too large for bot download?)"))?;

    // The download endpoint is /file/bot<token>/<file_path>.
    let dl_url = format!("{}/file/bot{}/{}", API_BASE, token, file_path);
    let bytes = client
        .get(&dl_url)
        .send()
        .await
        .context("file download request failed")?
        .error_for_status()
        .context("file download returned an error status")?
        .bytes()
        .await
        .context("reading downloaded file bytes")?;

    if bytes.len() as u64 > MAX_DOWNLOAD_BYTES {
        return Err(anyhow!(
            "downloaded {} bytes — over the {} byte intake cap",
            bytes.len(),
            MAX_DOWNLOAD_BYTES
        ));
    }
    Ok(bytes.to_vec())
}

// ── Voice transcription (OpenAI Whisper) ──

#[derive(Debug, Deserialize)]
struct WhisperResp {
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    error: Option<WhisperError>,
}

#[derive(Debug, Deserialize)]
struct WhisperError {
    #[serde(default)]
    message: Option<String>,
}

/// Transcribe an audio blob (Telegram voice = OGG/Opus) via the OpenAI
/// audio-transcriptions API. `api_key` is the stored `codex` (OPENAI) key;
/// the caller is responsible for the honest-degrade message when it's empty.
pub async fn transcribe_voice(
    client: &reqwest::Client,
    api_key: &str,
    audio: Vec<u8>,
) -> Result<String> {
    if api_key.is_empty() {
        return Err(anyhow!("no OpenAI API key"));
    }
    // Telegram voice notes are OGG/Opus; Whisper accepts "ogg".
    let part = reqwest::multipart::Part::bytes(audio)
        .file_name("voice.ogg")
        .mime_str("audio/ogg")
        .context("building audio multipart part")?;
    let form = reqwest::multipart::Form::new()
        .text("model", "whisper-1")
        .part("file", part);

    let resp = client
        .post("https://api.openai.com/v1/audio/transcriptions")
        .bearer_auth(api_key)
        .multipart(form)
        .send()
        .await
        .context("transcription request failed")?;

    let parsed: WhisperResp = resp
        .json()
        .await
        .context("transcription response decode failed")?;
    if let Some(err) = parsed.error {
        return Err(anyhow!(
            "OpenAI transcription error: {}",
            err.message.unwrap_or_else(|| "unknown".into())
        ));
    }
    parsed
        .text
        .filter(|t| !t.trim().is_empty())
        .ok_or_else(|| anyhow!("transcription returned empty text"))
}

// ── Photo vision (Anthropic Messages API) ──

#[derive(Debug, Deserialize)]
struct AnthropicResp {
    #[serde(default)]
    content: Vec<AnthropicContent>,
    #[serde(default)]
    error: Option<AnthropicError>,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    #[serde(default, rename = "type")]
    _type: String,
    #[serde(default)]
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnthropicError {
    #[serde(default)]
    message: Option<String>,
}

/// Describe an image via the Anthropic Messages vision API. `api_key` is the
/// stored `claude` key; the caller handles the honest-degrade message when it
/// is empty. `caption` (if any) is woven into the prompt so the model answers
/// the user's actual question about the image rather than describing blindly.
pub async fn describe_image(
    client: &reqwest::Client,
    api_key: &str,
    image: &[u8],
    media_type: &str,
    caption: Option<&str>,
) -> Result<String> {
    if api_key.is_empty() {
        return Err(anyhow!("no Anthropic API key"));
    }
    let b64 = base64_encode(image);
    let prompt = match caption {
        Some(c) if !c.trim().is_empty() => format!(
            "The user sent this image with the caption: \"{}\". \
             Answer their request about the image, or if there is no clear \
             request, describe the image precisely and note anything actionable.",
            c.trim()
        ),
        _ => "Describe this image precisely and concisely. Transcribe any text \
              you can read in it. Note anything that looks actionable (an error \
              message, a screenshot of code, a diagram, a chart)."
            .to_string(),
    };

    let payload = serde_json::json!({
        "model": "claude-3-5-sonnet-latest",
        "max_tokens": 1024,
        "messages": [{
            "role": "user",
            "content": [
                {
                    "type": "image",
                    "source": {
                        "type": "base64",
                        "media_type": media_type,
                        "data": b64,
                    }
                },
                { "type": "text", "text": prompt }
            ]
        }]
    });

    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&payload)
        .send()
        .await
        .context("vision request failed")?;

    let parsed: AnthropicResp = resp
        .json()
        .await
        .context("vision response decode failed")?;
    if let Some(err) = parsed.error {
        return Err(anyhow!(
            "Anthropic vision error: {}",
            err.message.unwrap_or_else(|| "unknown".into())
        ));
    }
    let text: String = parsed
        .content
        .into_iter()
        .filter_map(|c| c.text)
        .collect::<Vec<_>>()
        .join("\n");
    if text.trim().is_empty() {
        return Err(anyhow!("vision returned empty text"));
    }
    Ok(text)
}

// ── Document handling ──

/// Outcome of processing a downloaded document.
pub enum DocumentOutcome {
    /// The document was text-like; its UTF-8 content is returned inline.
    Text(String),
    /// The document was binary/large; it was saved here on disk.
    Saved { path: String, bytes: usize },
}

/// Decide whether a mime type is text-like enough to inline.
fn is_textual(mime: &str, file_name: &str) -> bool {
    let m = mime.to_ascii_lowercase();
    if m.starts_with("text/") {
        return true;
    }
    if matches!(
        m.as_str(),
        "application/json"
            | "application/xml"
            | "application/x-yaml"
            | "application/yaml"
            | "application/toml"
            | "application/javascript"
            | "application/x-sh"
            | "application/x-shellscript"
    ) {
        return true;
    }
    // Fall back to the extension when the mime is generic/unknown.
    let lower = file_name.to_ascii_lowercase();
    const TEXT_EXTS: &[&str] = &[
        ".txt", ".md", ".markdown", ".json", ".jsonl", ".yaml", ".yml", ".toml", ".csv", ".tsv",
        ".xml", ".html", ".css", ".js", ".ts", ".tsx", ".jsx", ".rs", ".py", ".go", ".rb", ".sh",
        ".c", ".h", ".cpp", ".hpp", ".java", ".kt", ".swift", ".sql", ".log", ".env", ".ini",
        ".cfg", ".conf",
    ];
    TEXT_EXTS.iter().any(|e| lower.ends_with(e))
}

/// Max bytes we will inline as text (a wall of text larger than this is saved
/// to disk instead, to avoid flooding the chat / a prompt).
const MAX_INLINE_TEXT_BYTES: usize = 200 * 1024;

/// Process a downloaded document blob: inline its text if it's text-like and
/// small enough, otherwise persist it to the local inbox and report the path.
pub fn process_document(bytes: Vec<u8>, file_name: &str, mime: &str) -> Result<DocumentOutcome> {
    let textual = is_textual(mime, file_name);
    if textual && bytes.len() <= MAX_INLINE_TEXT_BYTES {
        // Only inline when it actually decodes as UTF-8 — a mislabeled binary
        // shouldn't produce mojibake in the prompt.
        if let Ok(s) = String::from_utf8(bytes.clone()) {
            return Ok(DocumentOutcome::Text(s));
        }
    }
    let path = save_to_inbox(&bytes, file_name)?;
    Ok(DocumentOutcome::Saved {
        path,
        bytes: bytes.len(),
    })
}

/// Persist a downloaded blob under `~/.omega/state/telegram-inbox/`, returning
/// the absolute path. The filename is sanitized and timestamp-prefixed so two
/// uploads with the same name don't collide.
pub fn save_to_inbox(bytes: &[u8], file_name: &str) -> Result<String> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("no home dir"))?;
    let dir = home.join(".omega/state/telegram-inbox");
    std::fs::create_dir_all(&dir).context("creating telegram inbox dir")?;

    let safe: String = file_name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect();
    let safe = if safe.is_empty() { "file.bin".to_string() } else { safe };
    let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    let path = dir.join(format!("{}-{}", stamp, safe));
    std::fs::write(&path, bytes).with_context(|| format!("writing {}", path.display()))?;
    Ok(path.to_string_lossy().to_string())
}

// ── Self-contained base64 (no extra dependency) ──

/// Standard base64 (RFC 4648) encoder. Implemented locally to avoid pulling a
/// new crate into the workspace just for the vision image payload.
fn base64_encode(input: &[u8]) -> String {
    const TABLE: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((n >> 18) & 0x3F) as usize] as char);
        out.push(TABLE[((n >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((n >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(n & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_known_vectors() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
        assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn textual_detection() {
        assert!(is_textual("text/plain", "notes"));
        assert!(is_textual("application/json", "x"));
        assert!(is_textual("application/octet-stream", "config.toml"));
        assert!(!is_textual("application/octet-stream", "photo.bin"));
        assert!(!is_textual("application/pdf", "report.pdf"));
    }
}
