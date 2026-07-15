pub mod app;
pub mod config;
pub mod context;
pub mod controller;
pub mod diff;
pub mod finder;
pub mod git;
pub mod graph;
pub mod help;
pub mod host;
pub mod input;
pub mod intent;
pub mod launch;
pub mod picker;
pub mod presenter;
pub mod root;

pub use app::run;

pub const PANE_LABEL: &str = "Git Graph";
