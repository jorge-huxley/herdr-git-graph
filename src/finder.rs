use crate::graph::GraphRow;

#[derive(Default)]
pub struct FinderState {
    pub open: bool,
    pub query: String,
    pub matches: Vec<usize>,
    pub cursor: usize,
}

impl FinderState {
    pub fn open(&mut self) {
        self.open = true;
        self.query.clear();
        self.matches.clear();
        self.cursor = 0;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn push_char(&mut self, c: char, rows: &[GraphRow]) {
        self.query.push(c);
        self.refresh(rows);
    }

    pub fn backspace(&mut self, rows: &[GraphRow]) {
        self.query.pop();
        self.refresh(rows);
    }

    pub fn move_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor + 1 < self.matches.len() {
            self.cursor += 1;
        }
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.matches.get(self.cursor).copied()
    }

    fn refresh(&mut self, rows: &[GraphRow]) {
        let q = self.query.to_lowercase();
        self.matches = rows
            .iter()
            .enumerate()
            .filter(|(_, r)| {
                r.subject.to_lowercase().contains(&q)
                    || r.hash.to_lowercase().contains(&q)
                    || r.refs.to_lowercase().contains(&q)
            })
            .map(|(i, _)| i)
            .collect();
        if self.cursor >= self.matches.len() {
            self.cursor = self.matches.len().saturating_sub(1);
        }
    }
}
