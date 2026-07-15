#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitNode {
    pub hash: String,
    pub parents: Vec<String>,
    pub refs: Vec<String>,
    pub subject: String,
    pub author: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum BranchFilter {
    #[default]
    All,
    LocalOnly,
    Branch(String),
}

impl BranchFilter {
    pub fn label(&self) -> String {
        match self {
            BranchFilter::All => "all branches".to_string(),
            BranchFilter::LocalOnly => "local branches".to_string(),
            BranchFilter::Branch(b) => b.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommitGraph {
    pub commits: Vec<CommitNode>,
    pub rows: Vec<GraphRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphRow {
    pub graph: String,
    pub short_hash: String,
    pub refs: String,
    pub subject: String,
    pub hash: String,
}
