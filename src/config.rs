use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub commit_limit: usize,
    pub delta_command: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            commit_limit: 500,
            delta_command: None,
        }
    }
}

#[derive(Debug, Deserialize, Default)]
struct FileConfig {
    commit_limit: Option<usize>,
    delta_command: Option<String>,
}

pub fn load_config() -> Config {
    let mut cfg = Config::default();
    if let Some(path) = config_path() {
        if let Ok(text) = std::fs::read_to_string(&path) {
            if let Ok(file) = toml::from_str::<FileConfig>(&text) {
                if let Some(n) = file.commit_limit {
                    cfg.commit_limit = n.clamp(50, 5000);
                }
                cfg.delta_command = file.delta_command.filter(|s| !s.is_empty());
            }
        }
    }
    if cfg.delta_command.is_none() {
        if which::which("delta").is_ok() {
            cfg.delta_command = Some("delta".to_string());
        }
    }
    cfg
}

fn config_path() -> Option<PathBuf> {
    std::env::var("HERDR_PLUGIN_CONFIG_DIR")
        .ok()
        .map(PathBuf::from)
        .map(|d| d.join("config.toml"))
        .filter(|p| p.exists())
}

mod which {
    use std::process::Command;

    pub fn which(name: &str) -> Result<(), ()> {
        let cmd = if cfg!(windows) { "where" } else { "which" };
        Command::new(cmd)
            .arg(name)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .ok_or(())
    }
}
