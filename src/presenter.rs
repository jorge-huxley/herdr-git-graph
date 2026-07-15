use crate::controller::{Controller, Modal};
use crate::input::help_text;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

pub fn draw(frame: &mut Frame, ctrl: &Controller) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(3)])
        .split(area);

    let header = format!(
        " {} · {} · {}",
        ctrl.resolved.repo_name,
        ctrl.status,
        if ctrl.show_diff {
            "diff"
        } else {
            "details"
        }
    );
    frame.render_widget(
        Paragraph::new(header).style(Style::default().add_modifier(Modifier::BOLD)),
        chunks[0],
    );

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(chunks[1]);

    draw_graph_list(frame, ctrl, body[0]);
    draw_details(frame, ctrl, body[1]);

    match ctrl.modal {
        Modal::Help => draw_help(frame, area),
        Modal::BranchPicker => draw_picker(frame, ctrl, area, "Branch filter"),
        Modal::Search => draw_search(frame, ctrl, area),
        Modal::None => {}
    }
}

fn draw_graph_list(frame: &mut Frame, ctrl: &Controller, area: Rect) {
    let items: Vec<ListItem> = ctrl
        .rows
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let refs = if row.refs.is_empty() {
                String::new()
            } else {
                format!(" ({})", row.refs)
            };
            let line = format!(
                "{:<12} {:7} {}{}",
                row.graph, row.short_hash, row.subject, refs
            );
            let style = if i == ctrl.selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(" Commits ")
            .borders(Borders::ALL),
    );
    frame.render_widget(list, area);
}

fn draw_details(frame: &mut Frame, ctrl: &Controller, area: Rect) {
    let text = if ctrl.show_diff {
        if ctrl.diff_text.is_empty() {
            "Loading diff…".to_string()
        } else {
            ctrl.diff_text.clone()
        }
    } else {
        ctrl.details_text.clone()
    };

    let para = Paragraph::new(text)
        .wrap(Wrap { trim: false })
        .scroll((ctrl.details_scroll, 0))
        .block(
            Block::default()
                .title(if ctrl.show_diff {
                    " Diff "
                } else {
                    " Commit "
                })
                .borders(Borders::ALL),
        );
    frame.render_widget(para, area);
}

fn draw_help(frame: &mut Frame, area: Rect) {
    let popup = centered_rect(60, 40, area);
    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(help_text())
            .block(
                Block::default()
                    .title(" Help ")
                    .borders(Borders::ALL),
            )
            .wrap(Wrap { trim: true }),
        popup,
    );
}

fn draw_picker(frame: &mut Frame, ctrl: &Controller, area: Rect, title: &str) {
    let popup = centered_rect(50, 60, area);
    frame.render_widget(Clear, popup);
    let items: Vec<ListItem> = ctrl
        .picker
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == ctrl.picker.cursor {
                Style::default().reverse()
            } else {
                Style::default()
            };
            ListItem::new(item.clone()).style(style)
        })
        .collect();
    frame.render_widget(
        List::new(items).block(
            Block::default()
                .title(format!(" {title} "))
                .borders(Borders::ALL),
        ),
        popup,
    );
}

fn draw_search(frame: &mut Frame, ctrl: &Controller, area: Rect) {
    let popup = centered_rect(60, 50, area);
    frame.render_widget(Clear, popup);
    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(3)])
        .split(popup);

    frame.render_widget(
        Paragraph::new(format!("Search: {}", ctrl.finder.query)).block(
            Block::default()
                .title(" Search ")
                .borders(Borders::ALL),
        ),
        inner[0],
    );

    let items: Vec<ListItem> = ctrl
        .finder
        .matches
        .iter()
        .enumerate()
        .map(|(i, &row_idx)| {
            let row = &ctrl.rows[row_idx];
            let style = if i == ctrl.finder.cursor {
                Style::default().reverse()
            } else {
                Style::default()
            };
            ListItem::new(format!("{} {}", row.short_hash, row.subject)).style(style)
        })
        .collect();
    frame.render_widget(
        List::new(items).block(Block::default().borders(Borders::ALL)),
        inner[1],
    );
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
