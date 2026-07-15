#[derive(Default)]
pub struct HelpState {
    pub open: bool,
}

impl HelpState {
    pub fn toggle(&mut self) {
        self.open = !self.open;
    }

    pub fn close(&mut self) {
        self.open = false;
    }
}
