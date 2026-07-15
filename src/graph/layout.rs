use super::model::{CommitNode, GraphRow};

const EMPTY: char = ' ';
const PIPE: char = '│';
const MERGE_R: char = '├';
const MERGE_L: char = '└';
const BRANCH: char = '┬';
const HORIZ: char = '─';
const COMMIT: char = '●';

pub fn layout_graph(commits: &[CommitNode]) -> Vec<GraphRow> {
    if commits.is_empty() {
        return Vec::new();
    }

    let n = commits.len();
    let mut id_to_idx = std::collections::HashMap::new();
    for (i, c) in commits.iter().enumerate() {
        id_to_idx.insert(c.hash.as_str(), i);
    }

    // Active lanes track which commit index each column leads toward (downward).
    let mut lanes: Vec<Option<usize>> = Vec::new();
    let mut rows = Vec::with_capacity(n);

    for (idx, commit) in commits.iter().enumerate() {
        // Locate or allocate the lane for this commit.
        let lane = lanes
            .iter()
            .position(|l| *l == Some(idx))
            .or_else(|| lanes.iter().position(|l| l.is_none()))
            .unwrap_or_else(|| {
                lanes.push(None);
                lanes.len() - 1
            });

        while lanes.len() <= lane {
            lanes.push(None);
        }

        let mut cols: Vec<char> = vec![EMPTY; lanes.len()];
        for (i, slot) in lanes.iter().enumerate() {
            if slot.is_some() && i != lane {
                cols[i] = PIPE;
            }
        }

        // Parent lanes to activate after this row.
        let mut parent_lanes: Vec<usize> = Vec::new();
        for (pi, parent) in commit.parents.iter().enumerate() {
            if let Some(&parent_idx) = id_to_idx.get(parent.as_str()) {
                let pl = if pi == 0 {
                    lane
                } else {
                    lanes
                        .iter()
                        .position(|l| l.is_none())
                        .unwrap_or_else(|| {
                            lanes.push(None);
                            lanes.len() - 1
                        })
                };
                while lanes.len() <= pl {
                    lanes.push(None);
                }
                while cols.len() <= pl {
                    cols.push(EMPTY);
                }
                parent_lanes.push(pl);
            }
        }

        // Draw connector glyphs at parent lane positions.
        if parent_lanes.len() > 1 {
            for (i, &pl) in parent_lanes.iter().enumerate() {
                if pl == lane {
                    cols[pl] = MERGE_R;
                } else if i + 1 == parent_lanes.len() {
                    cols[pl] = MERGE_L;
                } else {
                    cols[pl] = BRANCH;
                }
            }
        } else if parent_lanes.len() == 1 {
            cols[lane] = PIPE;
        }

        cols[lane] = COMMIT;

        let graph: String = cols
            .into_iter()
            .flat_map(|c| [c, HORIZ])
            .collect::<String>()
            .trim_end_matches(HORIZ)
            .to_string();

        let short_hash: String = commit.hash.chars().take(7).collect();
        rows.push(GraphRow {
            graph,
            short_hash,
            refs: commit.refs.join(", "),
            subject: commit.subject.clone(),
            hash: commit.hash.clone(),
        });

        // Advance lanes downward.
        lanes[lane] = None;
        for (pi, parent) in commit.parents.iter().enumerate() {
            if let Some(&parent_idx) = id_to_idx.get(parent.as_str()) {
                let pl = parent_lanes.get(pi).copied().unwrap_or(lane);
                if pl < lanes.len() {
                    lanes[pl] = Some(parent_idx);
                }
            }
        }

        while lanes.last().copied().flatten().is_none() && lanes.len() > 1 {
            lanes.pop();
        }
    }

    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::model::CommitNode;

    fn node(hash: &str, parents: &[&str], subject: &str) -> CommitNode {
        CommitNode {
            hash: hash.to_string(),
            parents: parents.iter().map(|s| s.to_string()).collect(),
            refs: Vec::new(),
            subject: subject.to_string(),
            author: "Test".to_string(),
            timestamp: 0,
        }
    }

    #[test]
    fn linear_history_has_rows() {
        let commits = vec![
            node("c3", &["c2"], "third"),
            node("c2", &["c1"], "second"),
            node("c1", &[], "first"),
        ];
        let rows = layout_graph(&commits);
        assert_eq!(rows.len(), 3);
        assert!(rows[0].graph.contains('●'));
        assert_eq!(rows[0].subject, "third");
    }

    #[test]
    fn merge_commit_produces_row() {
        let commits = vec![
            node("m", &["b", "a"], "merge"),
            node("b", &["r"], "branch"),
            node("a", &["r"], "main"),
            node("r", &[], "root"),
        ];
        let rows = layout_graph(&commits);
        assert_eq!(rows.len(), 4);
        assert!(rows.iter().all(|r| !r.graph.is_empty()));
    }

    #[test]
    fn empty_commits_empty_rows() {
        assert!(layout_graph(&[]).is_empty());
    }
}
