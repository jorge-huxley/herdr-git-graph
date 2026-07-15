use herdr_git_graph::{launch, PANE_LABEL};

#[test]
fn launch_decision_opens_when_no_viewer() {
    let json = r#"{"result":{"panes":[{"pane_id":"w1:p1","label":"","focused":true,"tab_id":"w1:t1"}]}}"#;
    assert_eq!(launch::launch_decision(json, PANE_LABEL), "OPEN");
}

#[test]
fn launch_decision_closes_focused_viewer() {
    let json = r#"{"result":{"panes":[
        {"pane_id":"w1:p1","label":"","focused":false,"tab_id":"w1:t1"},
        {"pane_id":"w1:p2","label":"Git Graph","focused":true,"tab_id":"w1:t1"}
    ]}}"#;
    assert_eq!(launch::launch_decision(json, PANE_LABEL), "CLOSE w1:p2");
}
