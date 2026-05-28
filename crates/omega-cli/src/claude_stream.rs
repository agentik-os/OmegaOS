//! Persistent Claude subprocess for instant Telegram responses.
//!
//! Spawns one `claude --output-format=stream-json --input-format=stream-json`
//! at bridge startup. Keeps stdin/stdout pipes open. Each user message =
//! one JSON write + read events until `{"type":"result"}`.
//!
//! First-message cost: ~3s (claude CLI startup). Subsequent messages:
//! dominated by LLM inference (streaming tokens). No 3s startup per message.

use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;

pub struct PersistentClaude {
    _child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl PersistentClaude {
    /// Spawn the persistent claude subprocess as the AISB Master:
    /// - cwd = $HOME (full system access, not scoped to any project)
    /// - --append-system-prompt-file loads the AISB Master prompt
    ///   (router/dispatcher identity — NEVER does the work itself)
    pub async fn spawn(config_dir: Option<PathBuf>) -> Result<Self> {
        let mut cmd = Command::new("claude");
        if let Some(dir) = config_dir {
            cmd.env("CLAUDE_CONFIG_DIR", dir);
        }

        // AISB Master runs from $HOME — full system access, all projects in scope.
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        cmd.current_dir(&home);

        // Inject the AISB Master system prompt so the model knows it's the
        // dispatcher, not a project agent.
        let aisb_prompt = home.join(".omega/agents/aisb-master.md");
        let mut args: Vec<&str> = vec![
            "--print",
            "--output-format=stream-json",
            "--input-format=stream-json",
            "--dangerously-skip-permissions",
            "--verbose",
        ];
        let prompt_arg;
        if aisb_prompt.exists() {
            prompt_arg = aisb_prompt.to_string_lossy().to_string();
            args.push("--append-system-prompt-file");
            args.push(&prompt_arg);
        }
        cmd.args(&args);
        cmd.stdin(std::process::Stdio::piped());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::null());

        let mut child = cmd.spawn().context("spawn claude stream subprocess")?;
        let stdin = child.stdin.take().context("no stdin pipe")?;
        let stdout = child.stdout.take().context("no stdout pipe")?;

        Ok(Self {
            _child: child,
            stdin,
            stdout: BufReader::new(stdout),
        })
    }

    /// Send a prompt, read events until a `result` event, return the text.
    pub async fn ask(&mut self, prompt: &str) -> Result<String> {
        let msg = serde_json::json!({
            "type": "user",
            "message": { "role": "user", "content": prompt }
        });
        let mut line = serde_json::to_string(&msg)?;
        line.push('\n');
        self.stdin.write_all(line.as_bytes()).await?;
        self.stdin.flush().await?;

        // Read events until we see a `result` event.
        loop {
            let mut buf = String::new();
            let n = self.stdout.read_line(&mut buf).await?;
            if n == 0 {
                anyhow::bail!("claude subprocess closed stdout");
            }
            let val: serde_json::Value = match serde_json::from_str(&buf) {
                Ok(v) => v,
                Err(_) => continue, // skip non-JSON lines
            };
            let t = val.get("type").and_then(|v| v.as_str()).unwrap_or("");
            if t == "result" {
                let result = val
                    .get("result")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                return Ok(result);
            }
        }
    }
}

/// Shared handle: lazy init on first use, kept alive for the bridge lifetime.
pub struct ClaudeStreamHandle {
    inner: Mutex<Option<PersistentClaude>>,
    config_dir: Option<PathBuf>,
}

impl ClaudeStreamHandle {
    pub fn new(config_dir: Option<PathBuf>) -> Self {
        Self {
            inner: Mutex::new(None),
            config_dir,
        }
    }

    /// Ask claude. Lazy-spawns on first call. On subprocess crash, respawns.
    pub async fn ask(&self, prompt: &str) -> Result<String> {
        let mut guard = self.inner.lock().await;
        if guard.is_none() {
            *guard = Some(PersistentClaude::spawn(self.config_dir.clone()).await?);
        }
        let claude = guard.as_mut().unwrap();
        match claude.ask(prompt).await {
            Ok(response) => Ok(response),
            Err(e) => {
                // Subprocess died — drop it so the next call respawns.
                *guard = None;
                Err(e)
            }
        }
    }
}
