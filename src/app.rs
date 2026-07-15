use crate::config;
use crate::controller::Controller;
use crate::host;
use crate::input;
use crate::intent::Intent;
use crate::presenter;
use crate::root;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self, stdout, Stdout};
use std::time::Duration;

pub fn run() -> io::Result<()> {
    let ctx = host::from_env();
    let resolved = root::resolve(&ctx);
    let cfg = config::load_config();
    let mut ctrl = Controller::new(resolved, cfg);

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let result = event_loop(&mut terminal, &mut ctrl);

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    result
}

fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ctrl: &mut Controller,
) -> io::Result<()> {
    loop {
        ctrl.poll_diff();
        terminal.draw(|f| presenter::draw(f, ctrl))?;

        if ctrl.should_quit {
            break;
        }

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if key.code == KeyCode::Backspace && matches!(ctrl.modal, crate::controller::Modal::Search) {
                        ctrl.finder.backspace(&ctrl.rows);
                        continue;
                    }
                    if let KeyCode::Char(c) = key.code {
                        if matches!(ctrl.modal, crate::controller::Modal::Search)
                            && !c.is_control()
                        {
                            ctrl.handle_key_char(c);
                            continue;
                        }
                    }
                    if let Some(intent) = input::map_key(key) {
                        if matches!(ctrl.modal, crate::controller::Modal::BranchPicker | crate::controller::Modal::Search)
                            && key.code == KeyCode::Enter
                        {
                            ctrl.handle(Intent::Confirm);
                        } else if matches!(intent, Intent::Confirm) {
                            ctrl.handle(Intent::Confirm);
                        } else if matches!(intent, Intent::Cancel)
                            && matches!(ctrl.modal, crate::controller::Modal::None)
                        {
                            ctrl.should_quit = true;
                        } else {
                            ctrl.handle(intent);
                        }
                    }
                }
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
    }
    Ok(())
}
