use serde::Deserialize;

#[derive(Deserialize)]
struct PaneList {
    result: PaneListResult,
}

#[derive(Deserialize)]
struct PaneListResult {
    #[serde(default)]
    panes: Vec<Pane>,
}

#[derive(Deserialize)]
struct Pane {
    pane_id: Option<String>,
    label: Option<String>,
    #[serde(default)]
    focused: bool,
    tab_id: Option<String>,
}

pub fn launch_decision(pane_list_json: &str, pane_label: &str) -> String {
    let Ok(list) = serde_json::from_str::<PaneList>(pane_list_json) else {
        return "OPEN".to_string();
    };
    let panes = &list.result.panes;
    let Some(focused) = panes.iter().find(|p| p.focused) else {
        return "OPEN".to_string();
    };
    let tab = focused.tab_id.as_deref();
    let viewer = panes.iter().find(|p| {
        p.label.as_deref() == Some(pane_label) && p.tab_id.as_deref() == tab
    });
    let Some(viewer) = viewer else {
        return "OPEN".to_string();
    };
    let Some(id) = viewer.pane_id.as_deref().filter(|id| is_flag_safe(id)) else {
        return "OPEN".to_string();
    };
    if Some(id) == focused.pane_id.as_deref() {
        format!("CLOSE {id}")
    } else {
        format!("FOCUS {id}")
    }
}

pub fn launch_decision_tab(pane_list_json: &str, pane_label: &str) -> String {
    let Ok(list) = serde_json::from_str::<PaneList>(pane_list_json) else {
        return "OPEN".to_string();
    };
    let panes = &list.result.panes;
    let Some(focused) = panes.iter().find(|p| p.focused) else {
        return "OPEN".to_string();
    };
    let is_viewer = |p: &&Pane| p.label.as_deref() == Some(pane_label);

    if let Some(here) = panes.iter().find(|p| {
        is_viewer(p) && p.tab_id.as_deref() == focused.tab_id.as_deref()
    }) {
        let Some(id) = here.pane_id.as_deref().filter(|id| is_flag_safe(id)) else {
            return "OPEN".to_string();
        };
        return if Some(id) == focused.pane_id.as_deref() {
            format!("CLOSE {id}")
        } else {
            format!("FOCUS {id}")
        };
    }

    let focused_ws = workspace_of(focused);
    if let Some(ws) = focused_ws {
        if let Some(elsewhere) = panes
            .iter()
            .find(|p| is_viewer(p) && workspace_of(p) == Some(ws))
        {
            if let Some(tab) = elsewhere.tab_id.as_deref().filter(|t| is_flag_safe(t)) {
                return format!("SWITCHTAB {tab}");
            }
        }
    }
    "OPEN".to_string()
}

fn workspace_of(p: &Pane) -> Option<&str> {
    p.tab_id
        .as_deref()
        .and_then(|t| t.split_once(':'))
        .or_else(|| p.pane_id.as_deref().and_then(|t| t.split_once(':')))
        .map(|(ws, _)| ws)
        .filter(|w| !w.is_empty())
}

fn is_flag_safe(id: &str) -> bool {
    !id.is_empty()
        && !id.starts_with('-')
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | ':' | '.'))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pane(id: &str, label: &str, focused: bool, tab: &str) -> String {
        format!(
            r#"{{"pane_id":"{id}","label":"{label}","focused":{focused},"tab_id":"{tab}"}}"#
        )
    }

    fn list(panes: &[String]) -> String {
        format!(r#"{{"result":{{"panes":[{}]}}}}"#, panes.join(","))
    }

    #[test]
    fn no_viewer_pane_opens() {
        let j = list(&[pane("wE:p1", "", true, "wE:t1")]);
        assert_eq!(launch_decision(&j, "Git Graph"), "OPEN");
    }

    #[test]
    fn viewer_focused_closes() {
        let j = list(&[
            pane("wE:p1", "", false, "wE:t1"),
            pane("wE:pD", "Git Graph", true, "wE:t1"),
        ]);
        assert_eq!(launch_decision(&j, "Git Graph"), "CLOSE wE:pD");
    }

    #[test]
    fn tab_switches_to_other_tab() {
        let j = list(&[
            pane("wE:p1", "", true, "wE:t1"),
            pane("wE:pD", "Git Graph", false, "wE:t4"),
        ]);
        assert_eq!(launch_decision_tab(&j, "Git Graph"), "SWITCHTAB wE:t4");
    }
}
