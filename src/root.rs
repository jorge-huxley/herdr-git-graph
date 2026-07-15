use crate::context::LaunchContext;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolved {
    pub root: PathBuf,
    pub is_git_repo: bool,
    pub repo_root: Option<PathBuf>,
    pub repo_name: String,
}

pub fn resolve(ctx: &LaunchContext) -> Resolved {
    match git_output(&ctx.cwd, &["rev-parse", "--show-toplevel"]) {
        Some(toplevel) => {
            let repo_root = PathBuf::from(&toplevel);
            let repo_name = repo_root
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("repo")
                .to_string();
            Resolved {
                root: repo_root.clone(),
                is_git_repo: true,
                repo_root: Some(repo_root),
                repo_name,
            }
        }
        None => {
            let name = ctx
                .cwd
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("directory")
                .to_string();
            Resolved {
                root: ctx.cwd.clone(),
                is_git_repo: false,
                repo_root: None,
                repo_name: name,
            }
        }
    }
}

fn git_output(dir: &Path, args: &[&str]) -> Option<String> {
    let out = crate::git::git_command(dir, args).output().ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        None
    }
}
