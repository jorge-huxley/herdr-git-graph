use crate::context::LaunchContext;
use serde::Deserialize;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Deserialize, Default)]
struct RawContext {
    focused_pane_cwd: Option<String>,
    workspace_cwd: Option<String>,
    cwd: Option<String>,
}

pub fn from_env() -> LaunchContext {
    let json = std::env::var("HERDR_PLUGIN_CONTEXT_JSON").ok();
    let cwd = std::env::current_dir().unwrap_or_default();
    parse_context(json.as_deref(), cwd)
}

pub fn parse_context(json: Option<&str>, fallback_cwd: PathBuf) -> LaunchContext {
    let raw: RawContext = json
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    let cwd = raw
        .focused_pane_cwd
        .filter(|s| !s.is_empty())
        .or(raw.workspace_cwd.filter(|s| !s.is_empty()))
        .or(raw.cwd.filter(|s| !s.is_empty()))
        .map(PathBuf::from)
        .unwrap_or(fallback_cwd);
    LaunchContext { cwd }
}

/// Close the Herdr pane hosting this process (best-effort).
///
/// Without this, `q` exits the TUI but leaves an empty "Git Graph" shell pane,
/// so the next open keybind closes that leftover before opening again.
pub fn close_own_pane() {
    let Ok(pane_id) = std::env::var("HERDR_PANE_ID") else {
        return;
    };
    if !is_safe_id(&pane_id) {
        return;
    }
    let herdr = std::env::var("HERDR_BIN_PATH").unwrap_or_else(|_| "herdr".to_string());
    let _ = Command::new(herdr)
        .args(["pane", "close", &pane_id])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

fn is_safe_id(id: &str) -> bool {
    !id.is_empty()
        && !id.starts_with('-')
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | ':' | '.'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_focused_pane_cwd() {
        let ctx = parse_context(
            Some(r#"{"focused_pane_cwd":"/a","workspace_cwd":"/b","cwd":"/c"}"#),
            PathBuf::from("/fallback"),
        );
        assert_eq!(ctx.cwd, PathBuf::from("/a"));
    }

    #[test]
    fn malformed_json_falls_back() {
        let ctx = parse_context(Some("not json"), PathBuf::from("/fallback"));
        assert_eq!(ctx.cwd, PathBuf::from("/fallback"));
    }

    #[test]
    fn rejects_unsafe_pane_ids() {
        assert!(!is_safe_id(""));
        assert!(!is_safe_id("--close-all"));
        assert!(!is_safe_id("w1:p1;rm"));
        assert!(is_safe_id("w3:pP"));
    }
}
