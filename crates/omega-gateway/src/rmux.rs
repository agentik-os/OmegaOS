use anyhow::{bail, Result};
use std::path::PathBuf;
use std::process::Command;

pub fn rmux_bin() -> PathBuf {
    if let Ok(bin) = std::env::var("OMEGA_RMUX_BIN") {
        return PathBuf::from(bin);
    }
    dirs::home_dir().expect("no home dir").join(".local/bin/rmux")
}

fn run(args: &[&str]) -> Result<String> {
    let out = Command::new(rmux_bin()).args(args).output()?;
    if !out.status.success() {
        bail!("{}", String::from_utf8_lossy(&out.stderr).trim());
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

pub fn list_sessions() -> Result<Vec<String>> {
    let out = run(&["ls", "-F", "#S"])?;
    Ok(out.lines().map(str::to_string).filter(|l| !l.is_empty()).collect())
}

pub fn capture_pane(session: &str, lines: u32) -> Result<String> {
    let start = format!("-{lines}");
    run(&["capture-pane", "-p", "-t", session, "-S", &start])
}
