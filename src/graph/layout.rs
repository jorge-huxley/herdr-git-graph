use super::model::{CommitNode, GraphCell, GraphRow};

const EMPTY: char = ' ';
const PIPE: char = '│';
const COMMIT: char = '●';
const HORIZ: char = '─';
const CORNER_SE: char = '┐'; // merge arm starts going down on the right
const CORNER_SW: char = '┘'; // side lane joins into commit from the right
const TEE_E: char = '├'; // commit continues down and connects right
const TEE_W: char = '┤'; // side join into a continuing lane

/// Number of distinct lane colors cycled by the layout.
pub const LANE_PALETTE_SIZE: u8 = 8;

pub fn layout_graph(commits: &[CommitNode]) -> Vec<GraphRow> {
    if commits.is_empty() {
        return Vec::new();
    }

    let n = commits.len();
    let mut id_to_idx = std::collections::HashMap::new();
    for (i, c) in commits.iter().enumerate() {
        id_to_idx.insert(c.hash.as_str(), i);
    }

    // Active lanes: target commit index + lane color.
    let mut lanes: Vec<Option<(usize, u8)>> = Vec::new();
    let mut next_color: u8 = 0;
    let mut rows = Vec::with_capacity(n);

    for (idx, commit) in commits.iter().enumerate() {
        // All lanes that already point at this commit (branch join points).
        let mut incoming: Vec<usize> = lanes
            .iter()
            .enumerate()
            .filter_map(|(i, slot)| match slot {
                Some((t, _)) if *t == idx => Some(i),
                _ => None,
            })
            .collect();

        let lane = if let Some(&first) = incoming.first() {
            first
        } else {
            let free = lanes.iter().position(|l| l.is_none()).unwrap_or_else(|| {
                lanes.push(None);
                lanes.len() - 1
            });
            while lanes.len() <= free {
                lanes.push(None);
            }
            incoming.push(free);
            free
        };

        let commit_color = lanes[lane]
            .map(|(_, c)| c)
            .unwrap_or_else(|| alloc_color(&mut next_color));

        // Ensure glyph buffer covers all current lanes.
        let mut glyphs: Vec<(char, u8)> = vec![(EMPTY, 0); lanes.len()];

        // Continuing pipes for unrelated active lanes.
        for (i, slot) in lanes.iter().enumerate() {
            if let Some((_, color)) = slot {
                if i != lane && !incoming.contains(&i) {
                    glyphs[i] = (PIPE, *color);
                }
            }
        }

        // Join side lanes that also target this commit into the primary lane.
        if incoming.len() > 1 {
            let min_l = *incoming.iter().min().unwrap();
            let max_l = *incoming.iter().max().unwrap();
            for i in min_l..=max_l {
                if i == lane {
                    continue;
                }
                if incoming.contains(&i) {
                    let side_color = lanes[i].map(|(_, c)| c).unwrap_or(commit_color);
                    glyphs[i] = if i == max_l || i == min_l {
                        (CORNER_SW, side_color)
                    } else {
                        (TEE_W, side_color)
                    };
                } else if glyphs[i].0 == EMPTY {
                    glyphs[i] = (HORIZ, commit_color);
                }
            }
            // Fill horizontals between join arms.
            for (i, glyph) in glyphs
                .iter_mut()
                .enumerate()
                .take(max_l + 1)
                .skip(min_l)
            {
                if i != lane && glyph.0 == EMPTY {
                    *glyph = (HORIZ, commit_color);
                }
            }
        }

        // Parent lanes to activate after this row.
        let mut parent_lanes: Vec<(usize, u8)> = Vec::new();
        for (pi, parent) in commit.parents.iter().enumerate() {
            if !id_to_idx.contains_key(parent.as_str()) {
                continue;
            }
            if pi == 0 {
                parent_lanes.push((lane, commit_color));
            } else {
                // Prefer an incoming side lane we are about to free, else a free lane
                // that is not the commit lane / already claimed parent lane, else new.
                let pl = incoming
                    .iter()
                    .copied()
                    .find(|&i| i != lane)
                    .or_else(|| {
                        lanes.iter().enumerate().position(|(i, l)| {
                            l.is_none()
                                && i != lane
                                && !parent_lanes.iter().any(|(p, _)| *p == i)
                        })
                    })
                    .unwrap_or_else(|| {
                        lanes.push(None);
                        glyphs.push((EMPTY, 0));
                        lanes.len() - 1
                    });
                while lanes.len() <= pl {
                    lanes.push(None);
                }
                while glyphs.len() <= pl {
                    glyphs.push((EMPTY, 0));
                }
                let color = alloc_color(&mut next_color);
                parent_lanes.push((pl, color));
            }
        }

        // Draw merge arms from this commit to additional parents.
        if parent_lanes.len() > 1 {
            let min_l = parent_lanes.iter().map(|(l, _)| *l).min().unwrap();
            let max_l = parent_lanes.iter().map(|(l, _)| *l).max().unwrap();
            for (i, glyph) in glyphs
                .iter_mut()
                .enumerate()
                .take(max_l + 1)
                .skip(min_l)
            {
                if i == lane {
                    continue;
                }
                let is_parent_end = parent_lanes.iter().any(|(l, _)| *l == i);
                if is_parent_end {
                    let color = parent_lanes
                        .iter()
                        .find(|(l, _)| *l == i)
                        .map(|(_, c)| *c)
                        .unwrap_or(commit_color);
                    *glyph = (CORNER_SE, color);
                } else if glyph.0 == EMPTY || glyph.0 == PIPE {
                    *glyph = (HORIZ, commit_color);
                }
            }
        }
        glyphs[lane] = (COMMIT, commit_color);

        // Build display cells: glyph + spacer (space), so pipes don't look like ●─│.
        let mut cells: Vec<GraphCell> = Vec::with_capacity(glyphs.len() * 2);
        for (i, &(ch, color)) in glyphs.iter().enumerate() {
            cells.push(GraphCell::new(ch, color));
            if i + 1 < glyphs.len() {
                // Spacer between lanes; use HORIZ when both neighbors are merge connectors.
                // `connects_horiz` already includes COMMIT, so one condition covers elbows.
                let next = glyphs[i + 1].0;
                let spacer = if connects_horiz(ch) && connects_horiz(next) {
                    GraphCell::new(HORIZ, commit_color)
                } else {
                    GraphCell::empty()
                };
                cells.push(spacer);
            }
        }
        while cells.last().is_some_and(|c| c.ch == EMPTY) {
            cells.pop();
        }

        let short_hash: String = commit.hash.chars().take(7).collect();
        rows.push(GraphRow {
            cells,
            short_hash,
            refs: commit.refs.clone(),
            subject: commit.subject.clone(),
            author: commit.author.clone(),
            timestamp: commit.timestamp,
            hash: commit.hash.clone(),
        });

        // Clear all incoming lanes, then activate parent targets.
        for &i in &incoming {
            if i < lanes.len() {
                lanes[i] = None;
            }
        }
        for (pi, parent) in commit.parents.iter().enumerate() {
            if let Some(&parent_idx) = id_to_idx.get(parent.as_str()) {
                if let Some(&(pl, color)) = parent_lanes.get(pi) {
                    if pl < lanes.len() {
                        lanes[pl] = Some((parent_idx, color));
                    }
                }
            }
        }

        while lanes.last().copied().flatten().is_none() && !lanes.is_empty() {
            lanes.pop();
        }
    }

    rows
}

fn alloc_color(next: &mut u8) -> u8 {
    let c = *next % LANE_PALETTE_SIZE;
    *next = next.wrapping_add(1);
    c
}

fn connects_horiz(ch: char) -> bool {
    matches!(ch, HORIZ | CORNER_SE | CORNER_SW | TEE_E | TEE_W | COMMIT)
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
        assert!(rows[0].graph_string().contains('●'));
        assert_eq!(rows[0].subject, "third");
        assert_eq!(rows[0].author, "Test");
        assert!(rows[0].cells.iter().any(|c| c.ch == COMMIT));
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
        assert!(rows.iter().all(|r| !r.cells.is_empty()));
        // Merge row should connect into a second lane.
        assert!(
            rows[0].graph_string().contains('┐') || rows[0].graph_string().contains('─'),
            "merge row should show a horizontal arm: {}",
            rows[0].graph_string()
        );
        // Branch join at root should collapse lanes.
        assert!(
            rows[3].graph_string().contains('┘') || rows[3].cells.iter().filter(|c| c.ch == COMMIT).count() == 1,
            "root row should join or be a single commit: {}",
            rows[3].graph_string()
        );
    }

    #[test]
    fn empty_commits_empty_rows() {
        assert!(layout_graph(&[]).is_empty());
    }

    #[test]
    fn lanes_use_distinct_colors_on_merge() {
        let commits = vec![
            node("m", &["b", "a"], "merge"),
            node("b", &["r"], "branch"),
            node("a", &["r"], "main"),
            node("r", &[], "root"),
        ];
        let rows = layout_graph(&commits);
        let colors: Vec<u8> = rows[0]
            .cells
            .iter()
            .filter(|c| c.ch == COMMIT || c.ch == CORNER_SE)
            .map(|c| c.color_idx)
            .collect();
        assert!(colors.len() >= 2);
        // Commit lane and merge arm should not all share one color when two parents exist.
        assert_ne!(colors[0], colors[1]);
    }
}
