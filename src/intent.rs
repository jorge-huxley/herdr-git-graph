#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intent {
    MoveUp,
    MoveDown,
    PageUp,
    PageDown,
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
