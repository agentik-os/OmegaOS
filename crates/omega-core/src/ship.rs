//! Ship pipeline — build, commit, push, deploy, verify.
//!
//! Mirrors the live system's oracle-ship.sh as typed Rust with
//! async steps, freeze-on-failure, and structured results.

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Stdio;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipConfig {
    pub build_command: String,
    pub deploy_command: Option<String>,
    pub deploy_timeout_secs: u64,
    pub auto_rollback: bool,
    pub freeze_on_fail: bool,
}

impl Default for ShipConfig {
    fn default() -> Self {
        Self {
            build_command: "cargo build --release".to_string(),
            deploy_command: None,
            deploy_timeout_secs: 600,
            auto_rollback: false,
            freeze_on_fail: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipResult {
    pub result: ShipOutcome,
    pub commit: Option<String>,
    pub deploy_url: Option<String>,
    pub steps_completed: Vec<ShipStep>,
    pub error: Option<String>,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShipOutcome {
    Ok,
    Failed,
    Skipped,
    Frozen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipStep {
    pub name: String,
    pub passed: bool,
    pub output: Option<String>,
    pub duration_ms: u64,
}

pub struct ShipPipeline {
    project_dir: PathBuf,
    state_dir: PathBuf,
    config: ShipConfig,
}

impl ShipPipeline {
    pub fn new(project_dir: PathBuf, state_dir: PathBuf, config: ShipConfig) -> Self {
        Self {
            project_dir,
            state_dir,
            config,
        }
    }

    pub fn is_frozen(&self, project: &str) -> bool {
        self.freeze_path(project).exists()
    }

    fn freeze_path(&self, project: &str) -> PathBuf {
        self.state_dir.join(format!("ship-{}.frozen", project))
    }

    fn lock_path(&self, project: &str) -> PathBuf {
        self.state_dir.join(format!("ship-{}.lock", project))
    }

    pub async fn execute(&self, project: &str, commit_msg: &str) -> ShipResult {
        let started_at = Utc::now();
        let mut steps = Vec::new();

        if self.is_frozen(project) {
            return ShipResult {
                result: ShipOutcome::Frozen,
                commit: None,
                deploy_url: None,
                steps_completed: steps,
                error: Some("Ship pipeline frozen — resolve before retrying".to_string()),
                started_at,
                finished_at: Utc::now(),
            };
        }

        // Step 1: Build
        match self.run_step("build", &self.config.build_command).await {
            Ok(step) => {
                let passed = step.passed;
                steps.push(step);
                if !passed {
                    return self.fail(steps, "Build failed", started_at);
                }
            }
            Err(e) => {
                return self.fail(steps, &format!("Build error: {}", e), started_at);
            }
        }

        // Step 2: Git add + commit
        let add_cmd = "git add -A";
        match self.run_step("stage", add_cmd).await {
            Ok(step) => {
                steps.push(step);
            }
            Err(e) => {
                return self.fail(steps, &format!("Stage error: {}", e), started_at);
            }
        }

        let commit_cmd = format!("git commit -m '{}'", commit_msg.replace('\'', "'\\''"));
        let commit_hash = match self.run_step("commit", &commit_cmd).await {
            Ok(step) => {
                let passed = step.passed;
                steps.push(step);
                if passed {
                    self.get_commit_hash().await.ok()
                } else {
                    None
                }
            }
            Err(e) => {
                return self.fail(steps, &format!("Commit error: {}", e), started_at);
            }
        };

        // Step 3: Pull --rebase
        match self.run_step("rebase", "git pull --rebase").await {
            Ok(step) => {
                let passed = step.passed;
                steps.push(step);
                if !passed {
                    return self.fail(steps, "Rebase failed — resolve conflicts", started_at);
                }
            }
            Err(e) => {
                return self.fail(steps, &format!("Rebase error: {}", e), started_at);
            }
        }

        // Step 4: Push
        match self.run_step("push", "git push").await {
            Ok(step) => {
                let passed = step.passed;
                steps.push(step);
                if !passed {
                    return self.fail(steps, "Push failed", started_at);
                }
            }
            Err(e) => {
                return self.fail(steps, &format!("Push error: {}", e), started_at);
            }
        }

        // Step 5: Deploy (optional)
        let deploy_url = if let Some(ref deploy_cmd) = self.config.deploy_command {
            match self.run_step("deploy", deploy_cmd).await {
                Ok(step) => {
                    let passed = step.passed;
                    let output = step.output.clone();
                    steps.push(step);
                    if !passed {
                        if self.config.freeze_on_fail {
                            let _ = self.freeze(project);
                        }
                        return self.fail(steps, "Deploy failed", started_at);
                    }
                    output
                }
                Err(e) => {
                    if self.config.freeze_on_fail {
                        let _ = self.freeze(project);
                    }
                    return self.fail(steps, &format!("Deploy error: {}", e), started_at);
                }
            }
        } else {
            None
        };

        ShipResult {
            result: ShipOutcome::Ok,
            commit: commit_hash,
            deploy_url,
            steps_completed: steps,
            error: None,
            started_at,
            finished_at: Utc::now(),
        }
    }

    async fn run_step(&self, name: &str, command: &str) -> Result<ShipStep> {
        let start = std::time::Instant::now();
        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(&self.project_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context(format!("Failed to execute ship step '{}'", name))?;

        let duration_ms = start.elapsed().as_millis() as u64;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = if stderr.is_empty() {
            stdout.to_string()
        } else {
            format!("{}\n{}", stdout, stderr)
        };

        Ok(ShipStep {
            name: name.to_string(),
            passed: output.status.success(),
            output: if combined.trim().is_empty() {
                None
            } else {
                Some(combined.chars().take(2000).collect())
            },
            duration_ms,
        })
    }

    async fn get_commit_hash(&self) -> Result<String> {
        let output = tokio::process::Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .current_dir(&self.project_dir)
            .output()
            .await?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    fn freeze(&self, project: &str) -> Result<()> {
        let path = self.freeze_path(project);
        std::fs::write(
            &path,
            format!("Frozen at {}\n", Utc::now().format("%Y-%m-%dT%H:%M:%SZ")),
        )?;
        tracing::warn!(project = %project, "Ship pipeline FROZEN due to deploy failure");
        Ok(())
    }

    pub fn unfreeze(&self, project: &str) -> Result<()> {
        let path = self.freeze_path(project);
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }

    pub fn write_result(&self, project: &str, result: &ShipResult) -> Result<()> {
        let path = self.state_dir.join(format!("ship-{}.result.json", project));
        let content = serde_json::to_string_pretty(result)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    fn fail(&self, steps: Vec<ShipStep>, error: &str, started_at: DateTime<Utc>) -> ShipResult {
        ShipResult {
            result: ShipOutcome::Failed,
            commit: None,
            deploy_url: None,
            steps_completed: steps,
            error: Some(error.to_string()),
            started_at,
            finished_at: Utc::now(),
        }
    }
}
