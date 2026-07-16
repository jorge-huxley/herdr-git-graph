#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intent {
    MoveUp,
    MoveDown,
    PageUp,
    PageDown,
    ToggleDetailsPane, // open commit details (idempotent; Esc closes)
    ToggleDiff,
    BranchPicker,
    Search,
    Help,
    Quit,
    Cancel,
    Confirm,
    ScrollDetailsUp,
    ScrollDetailsDown,
}
