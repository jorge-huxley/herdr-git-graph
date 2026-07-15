use crate::config::Config;
use crate::diff::{format_commit_details, DiffWorker};
use crate::finder::FinderState;
use crate::git;
use crate::graph::{parse_log, BranchFilter, GraphRow};
use crate::graph::{layout_graph, CommitNode};
use crate::help::HelpState;
use crate::intent::Intent;
use crate::picker::PickerState;
use crate::root::Resolved;
use std::path::PathBuf;

pub enum Modal {
    None,
    Help,
    BranchPicker,
    Search,
}

pub struct Controller {
    pub repo: PathBuf,
    pub resolved: Resolved,
    pub config: Config,
    pub filter: BranchFilter,
    pub commits: Vec<CommitNode>,
    pub rows: Vec<GraphRow>,
    pub selected: usize,
    pub details_scroll: u16,
    pub show_diff: bool,
    pub details_text: String,
    pub diff_text: String,
    pub diff_seq: u64,
    pub pending_diff_seq: u64,
    pub modal: Modal,
    pub help: HelpState,
    pub picker: PickerState,
    pub finder: FinderState,
    pub status: String,
    pub should_quit: bool,
    diff_worker: DiffWorker,
}

impl Controller {
    pub fn new(resolved: Resolved, config: Config) -> Self {
        let repo = resolved
            .repo_root
            .clone()
            .unwrap_or_else(|| resolved.root.clone());
        let diff_worker = DiffWorker::new(repo.clone(), config.delta_command.clone());
        let mut ctrl = Self {
            repo,
            resolved,
            config,
            filter: BranchFilter::All,
            commits: Vec::new(),
            rows: Vec::new(),
            selected: 0,
            details_scroll: 0,
            show_diff: false,
            details_text: String::new(),
            diff_text: String::new(),
            diff_seq: 0,
            pending_diff_seq: 0,
            modal: Modal::None,
            help: HelpState::default(),
            picker: PickerState::default(),
            finder: FinderState::default(),
            status: String::new(),
            should_quit: false,
            diff_worker,
        };
        ctrl.reload_graph();
        ctrl
    }

    pub fn handle(&mut self, intent: Intent) {
        match self.modal {
            Modal::Help => self.handle_help(intent),
            Modal::BranchPicker => self.handle_branch_picker(intent),
            Modal::Search => self.handle_search(intent),
            Modal::None => self.handle_main(intent),
        }
    }

    pub fn handle_key_char(&mut self, c: char) {
        if matches!(self.modal, Modal::Search) {
            self.finder.push_char(c, &self.rows);
        }
    }

    pub fn poll_diff(&mut self) {
        if let Some(result) = self.diff_worker.poll() {
            if result.seq >= self.pending_diff_seq {
                self.diff_text = result.text;
                self.diff_seq = result.seq;
            }
        }
    }

    fn handle_main(&mut self, intent: Intent) {
        match intent {
            Intent::MoveUp => self.move_selection(-1),
            Intent::MoveDown => self.move_selection(1),
            Intent::PageUp => self.move_selection(-10),
            Intent::PageDown => self.move_selection(10),
            Intent::ToggleDiff => self.toggle_diff(),
            Intent::BranchPicker => self.open_branch_picker(),
            Intent::Search => {
                self.finder.open();
                self.modal = Modal::Search;
            }
            Intent::Help => {
                self.help.toggle();
                self.modal = if self.help.open {
                    Modal::Help
                } else {
                    Modal::None
                };
            }
            Intent::ScrollDetailsUp => {
                self.details_scroll = self.details_scroll.saturating_sub(1);
            }
            Intent::ScrollDetailsDown => self.details_scroll = self.details_scroll.saturating_add(1),
            Intent::Quit | Intent::Cancel => self.should_quit = true,
            _ => {}
        }
    }

    fn handle_help(&mut self, intent: Intent) {
        match intent {
            Intent::Help | Intent::Cancel | Intent::Quit => {
                self.help.close();
                self.modal = Modal::None;
            }
            _ => {}
        }
    }

    fn handle_branch_picker(&mut self, intent: Intent) {
        match intent {
            Intent::MoveUp => self.picker.move_up(),
            Intent::MoveDown => self.picker.move_down(),
            Intent::Confirm => {
                if let Some(sel) = self.picker.selected() {
                    self.filter = match sel {
                        "all branches" => BranchFilter::All,
                        "local branches" => BranchFilter::LocalOnly,
                        other => BranchFilter::Branch(other.to_string()),
                    };
                    self.reload_graph();
                }
                self.picker.close();
                self.modal = Modal::None;
            }
            Intent::Cancel | Intent::Quit => {
                self.picker.close();
                self.modal = Modal::None;
            }
            _ => {}
        }
    }

    fn handle_search(&mut self, intent: Intent) {
        match intent {
            Intent::MoveUp => self.finder.move_up(),
            Intent::MoveDown => self.finder.move_down(),
            Intent::Confirm => {
                if let Some(idx) = self.finder.selected_index() {
                    self.selected = idx;
                    self.refresh_details();
                }
                self.finder.close();
                self.modal = Modal::None;
            }
            Intent::Cancel | Intent::Quit => {
                self.finder.close();
                self.modal = Modal::None;
            }
            _ => {}
        }
    }

    fn move_selection(&mut self, delta: i32) {
        if self.rows.is_empty() {
            return;
        }
        let len = self.rows.len() as i32;
        let next = (self.selected as i32 + delta).clamp(0, len - 1);
        self.selected = next as usize;
        self.details_scroll = 0;
        self.refresh_details();
    }

    fn toggle_diff(&mut self) {
        self.show_diff = !self.show_diff;
        if self.show_diff {
            self.load_diff();
        }
    }

    fn open_branch_picker(&mut self) {
        let mut items = vec![
            "all branches".to_string(),
            "local branches".to_string(),
        ];
        items.extend(git::list_branches(&self.repo));
        self.picker.open_with_items(items);
        self.modal = Modal::BranchPicker;
    }

    fn reload_graph(&mut self) {
        if !self.resolved.is_git_repo {
            self.status = "Not a git repository.".to_string();
            self.commits.clear();
            self.rows.clear();
            return;
        }

        let raw = match &self.filter {
            BranchFilter::LocalOnly => git::log_commits_local(&self.repo, self.config.commit_limit),
            _ => {
                let branch_arg = match &self.filter {
                    BranchFilter::All => None,
                    BranchFilter::Branch(b) => Some(b.as_str()),
                    BranchFilter::LocalOnly => None,
                };
                git::log_commits(&self.repo, self.config.commit_limit, branch_arg)
            }
        };
        let Some(raw) = raw else {
            self.status = "Failed to read git log.".to_string();
            return;
        };

        self.commits = parse_log(&raw);
        self.rows = layout_graph(&self.commits);
        self.selected = 0;
        self.status = format!(
            "{} commits · filter: {}",
            self.rows.len(),
            self.filter.label()
        );
        self.refresh_details();
    }

    fn refresh_details(&mut self) {
        if let Some(row) = self.rows.get(self.selected) {
            self.details_text = format_commit_details(&self.repo, &row.hash);
            if self.show_diff {
                self.load_diff();
            }
        } else {
            self.details_text.clear();
            self.diff_text.clear();
        }
    }

    fn load_diff(&mut self) {
        if let Some(row) = self.rows.get(self.selected) {
            self.pending_diff_seq += 1;
            self.diff_worker
                .dispatch(self.pending_diff_seq, &row.hash);
        }
    }
}
