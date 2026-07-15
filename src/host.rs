use crate::context::LaunchContext;
use serde::Deserialize;
use std::path::PathBuf;

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
}
