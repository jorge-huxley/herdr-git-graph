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
    assert!(rows[0].graph.contains('●'));
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
    assert!(rows.iter().all(|r| !r.graph.is_empty()));
}

#[test]
fn parse_and_layout_integration() {
    let raw = "aaa\0\0(HEAD -> main)\0init\0Dev\01700000000\0bbb\0aaa\0\0second\0Dev\01700000001\0";
    let commits = parse_log(raw);
    assert_eq!(commits.len(), 2);
    let rows = layout_graph(&commits);
    assert_eq!(rows.len(), 2);
}
