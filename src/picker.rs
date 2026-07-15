pub struct PickerState {
    pub open: bool,
    pub items: Vec<String>,
    pub cursor: usize,
}

impl Default for PickerState {
    fn default() -> Self {
        Self {
            open: false,
            items: Vec::new(),
            cursor: 0,
        }
    }
}

impl PickerState {
    pub fn open_with_items(&mut self, items: Vec<String>) {
        self.open = true;
        self.items = items;
        self.cursor = 0;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn move_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor + 1 < self.items.len() {
            self.cursor += 1;
        }
    }

    pub fn selected(&self) -> Option<&str> {
        self.items.get(self.cursor).map(String::as_str)
    }
}
