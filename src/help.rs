pub struct HelpState {
    pub open: bool,
}

impl Default for HelpState {
    fn default() -> Self {
        Self { open: false }
    }
}

impl HelpState {
    pub fn toggle(&mut self) {
        self.open = !self.open;
    }

    pub fn close(&mut self) {
        self.open = false;
    }
}
