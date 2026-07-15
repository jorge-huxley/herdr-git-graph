use std::path::Path;
use std::process::{Command, Stdio};

/// Build a hardened read-only `git` command rooted at `dir`.
pub fn git_command(dir: &Path, args: &[&str]) -> Command {
    let mut cmd = Command::new("git");
    cmd.current_dir(dir)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_OBJECT_DIRECTORY")
        .env_remove("GIT_ALTERNATE_OBJECT_DIRECTORIES")
        .env_remove("GIT_CEILING_DIRECTORIES")
        .env_remove("GIT_DISCOVERY_ACROSS_FILESYSTEM")
        .env("GIT_CONFIG_COUNT", "3")
        .env("GIT_CONFIG_KEY_0", "core.hooksPath")
        .env("GIT_CONFIG_VALUE_0", "/dev/null")
        .env("GIT_CONFIG_KEY_1", "core.fsmonitor")
        .env("GIT_CONFIG_VALUE_1", "")
        .env("GIT_CONFIG_KEY_2", "diff.external")
        .env("GIT_CONFIG_VALUE_2", "");
    cmd
}

pub fn run_git(dir: &Path, args: &[&str]) -> Option<String> {
    let out = git_command(dir, args).output().ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).to_string())
    } else {
        None
    }
}

pub fn list_branches(dir: &Path) -> Vec<String> {
    let Some(out) = run_git(
        dir,
        &[
            "for-each-ref",
            "--format=%(refname:short)",
            "refs/heads",
            "refs/remotes",
        ],
    ) else {
        return Vec::new();
    };
    out.lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

pub fn show_commit(dir: &Path, hash: &str) -> Option<String> {
    run_git(
        dir,
        &[
            "show",
            "--format=fuller",
            "--no-ext-diff",
            "--no-color",
            hash,
        ],
    )
}

pub fn show_diff(dir: &Path, hash: &str) -> Option<String> {
    run_git(
        dir,
        &["show", "--format=", "--no-ext-diff", "--no-color", "--patch", hash],
    )
}

pub fn log_commits_local(dir: &Path, limit: usize) -> Option<String> {
    let limit_str = limit.to_string();
    run_git(
        dir,
        &[
            "log",
            "refs/heads",
            "--topo-order",
            "--date-order",
            "-n",
            &limit_str,
            "--pretty=format:%H\x00%P\x00%d\x00%s\x00%an\x00%at\x00",
            "--encoding=UTF-8",
        ],
    )
}

pub fn log_commits(dir: &Path, limit: usize, branch: Option<&str>) -> Option<String> {
    let limit_str = limit.to_string();
    let mut args: Vec<&str> = vec![
        "log",
        "--topo-order",
        "--date-order",
        "-n",
        &limit_str,
        "--pretty=format:%H\x00%P\x00%d\x00%s\x00%an\x00%at\x00",
        "--encoding=UTF-8",
    ];
    if let Some(b) = branch {
        args.push(b);
    } else {
        args.push("--all");
    }
    run_git(dir, &args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command as StdCommand;
    use tempfile::TempDir;

    fn init_repo(dir: &Path) {
        StdCommand::new("git")
            .args(["init", "-b", "main"])
            .current_dir(dir)
            .output()
            .unwrap();
        StdCommand::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(dir)
            .output()
            .unwrap();
        StdCommand::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(dir)
            .output()
            .unwrap();
    }

    #[test]
    fn log_commits_reads_history() {
        let tmp = TempDir::new().unwrap();
        init_repo(tmp.path());
        std::fs::write(tmp.path().join("f.txt"), "hello").unwrap();
        StdCommand::new("git")
            .args(["add", "f.txt"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        StdCommand::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        let out = log_commits(tmp.path(), 10, None).unwrap();
        assert!(out.contains("initial"));
    }
}
