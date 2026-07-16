use herdr_git_graph::graph::{layout_graph, model::CommitNode, parse_log};

fn node(hash: &str, parents: &[&str], subject: &str, refs: &[&str]) -> CommitNode {
    CommitNode {
        hash: hash.to_string(),
        parents: parents.iter().map(|s| s.to_string()).collect(),
        refs: refs.iter().map(|s| s.to_string()).collect(),
        subject: subject.to_string(),
        author: "Author".to_string(),
        timestamp: 1_700_000_000,
    }
}

#[test]
fn layout_linear_history() {
    let commits = vec![
        node("c3", &["c2"], "third", &[]),
        node("c2", &["c1"], "second", &[]),
        node("c1", &[], "first", &["main"]),
    ];
    let rows = layout_graph(&commits);
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].subject, "third");
    assert!(rows[0].graph_string().contains('●'));
    assert_eq!(rows[0].author, "Author");
    assert_eq!(rows[0].timestamp, 1_700_000_000);
    assert_eq!(rows[2].refs, vec!["main"]);
    // Linear history stays on one lane color.
    let colors: Vec<_> = rows
        .iter()
        .flat_map(|r| r.cells.iter())
        .filter(|c| c.ch == '●')
        .map(|c| c.color_idx)
        .collect();
    assert_eq!(colors.len(), 3);
    assert!(colors.iter().all(|&c| c == colors[0]));
}

#[test]
fn layout_merge_history() {
    let commits = vec![
        node("m", &["b", "a"], "merge commit", &["main"]),
        node("b", &["r"], "feature work", &["feat"]),
        node("a", &["r"], "main work", &[]),
        node("r", &[], "root", &[]),
    ];
    let rows = layout_graph(&commits);
    assert_eq!(rows.len(), 4);
    assert!(rows.iter().all(|r| !r.cells.is_empty()));
    assert!(
        rows[0].graph_string().contains('┐') || rows[0].graph_string().contains('─'),
        "merge should show an arm: {}",
        rows[0].graph_string()
    );
    // Side branch and first parent should use different colors on the merge row.
    let merge_colors: Vec<u8> = rows[0]
        .cells
        .iter()
        .filter(|c| c.ch == '●' || c.ch == '┐')
        .map(|c| c.color_idx)
        .collect();
    assert!(merge_colors.len() >= 2);
    assert_ne!(merge_colors[0], merge_colors[1]);
    // Join at the common ancestor should collapse or corner.
    let root = rows[3].graph_string();
    assert!(
        root.contains('●'),
        "root must have a commit glyph: {root}"
    );
}

#[test]
fn layout_preserves_refs_as_vec() {
    let commits = vec![node(
        "c1",
        &[],
        "init",
        &["HEAD -> main", "origin/main", "tag: v1.0"],
    )];
    let rows = layout_graph(&commits);
    assert_eq!(
        rows[0].refs,
        vec![
            "HEAD -> main".to_string(),
            "origin/main".to_string(),
            "tag: v1.0".to_string()
        ]
    );
}

#[test]
fn parse_and_layout_integration() {
    // Root commit has empty parents field (`\0\0`); timestamps via concat to avoid `\0NN` octal.
    let raw = concat!(
        "aaa\0\0(HEAD -> main)\0init\0Dev\0",
        "1700000000\0",
        "bbb\0aaa\0\0second\0Dev\0",
        "1700000001\0",
    );
    let commits = parse_log(raw);
    assert_eq!(commits.len(), 2);
    assert_eq!(commits[0].refs, vec!["HEAD -> main"]);
    let rows = layout_graph(&commits);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].refs, vec!["HEAD -> main"]);
    assert!(!rows[0].cells.is_empty());
}
