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

/// One glyph in the left-hand ASCII graph, with a palette index for coloring.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphCell {
    pub ch: char,
    /// Index into the lane color palette (`0` = default / empty).
    pub color_idx: u8,
}

impl GraphCell {
    pub fn empty() -> Self {
        Self {
            ch: ' ',
            color_idx: 0,
        }
    }

    pub fn new(ch: char, color_idx: u8) -> Self {
        Self { ch, color_idx }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphRow {
    pub cells: Vec<GraphCell>,
    pub short_hash: String,
    pub refs: Vec<String>,
    pub subject: String,
    pub author: String,
    pub timestamp: i64,
    pub hash: String,
}

impl GraphRow {
    /// Flat graph string for tests and debugging.
    pub fn graph_string(&self) -> String {
        self.cells.iter().map(|c| c.ch).collect()
    }
}
