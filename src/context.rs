use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LaunchContext {
    pub cwd: PathBuf,
}
