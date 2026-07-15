use crate::intent::Intent;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn map_key(event: KeyEvent) -> Option<Intent> {
    match (event.code, event.modifiers) {
        (KeyCode::Char('q'), _) | (KeyCode::Esc, _) => Some(Intent::Cancel),
        (KeyCode::Char('j'), _) | (KeyCode::Down, _) => Some(Intent::MoveDown),
        (KeyCode::Char('k'), _) | (KeyCode::Up, _) => Some(Intent::MoveUp),
        (KeyCode::Char('u'), KeyModifiers::CONTROL) => Some(Intent::ScrollDetailsUp),
        (KeyCode::Char('d'), KeyModifiers::CONTROL) => Some(Intent::ScrollDetailsDown),
        (KeyCode::Char('d'), _) | (KeyCode::Enter, _) => Some(Intent::ToggleDiff),
        (KeyCode::Char('b'), _) => Some(Intent::BranchPicker),
        (KeyCode::Char('/'), _) => Some(Intent::Search),
        (KeyCode::Char('?'), _) => Some(Intent::Help),
        (KeyCode::PageDown, _) => Some(Intent::PageDown),
        (KeyCode::PageUp, _) => Some(Intent::PageUp),
        _ => None,
    }
}

pub fn help_text() -> &'static str {
    "j/k or arrows  move selection\n\
     Enter/d        show commit diff\n\
     b              branch filter\n\
     /              search commits\n\
     Ctrl-u/d       scroll details\n\
     ?              this help\n\
     q/Esc          close"
}
