use crate::controller::{Controller, Modal};
use crate::graph::layout::LANE_PALETTE_SIZE;
use crate::graph::GraphRow;
use crate::input::help_text;
use chrono::{Local, TimeZone, Utc};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

pub fn draw(frame: &mut Frame, ctrl: &Controller) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(3)])
        .split(area);

    let mode = if !ctrl.show_details_pane {
        "graph"
    } else if ctrl.show_diff {
        "diff"
    } else {
        "details"
    };
    let header = format!(" {} · {} · {}", ctrl.resolved.repo_name, ctrl.status, mode);
    frame.render_widget(
        Paragraph::new(header).style(Style::default().add_modifier(Modifier::BOLD)),
        chunks[0],
    );

    if ctrl.show_details_pane {
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
            .split(chunks[1]);
        draw_graph_list(frame, ctrl, body[0]);
        draw_details(frame, ctrl, body[1]);
    } else {
        draw_graph_list(frame, ctrl, chunks[1]);
    }

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
            let selected = i == ctrl.selected;
            ListItem::new(commit_line(row, selected))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(" Commits ")
            .borders(Borders::ALL),
    );
    frame.render_widget(list, area);
}

fn commit_line(row: &GraphRow, selected: bool) -> Line<'static> {
    let sel_bg = if selected {
        Some(Color::DarkGray)
    } else {
        None
    };

    let mut spans: Vec<Span<'static>> = Vec::new();

    // Colored ASCII graph.
    for cell in &row.cells {
        let mut style = Style::default().fg(lane_color(cell.color_idx));
        if let Some(bg) = sel_bg {
            style = style.bg(bg);
        }
        if cell.ch == '●' {
            style = style.add_modifier(Modifier::BOLD);
        }
        spans.push(Span::styled(cell.ch.to_string(), style));
    }
    spans.push(pad_span("  ", sel_bg));

    // Ref pills.
    for rf in &row.refs {
        spans.push(ref_pill(rf, sel_bg));
        spans.push(pad_span(" ", sel_bg));
    }

    // Subject.
    let mut subject_style = Style::default();
    if let Some(bg) = sel_bg {
        subject_style = subject_style.bg(bg).add_modifier(Modifier::BOLD);
    }
    spans.push(Span::styled(row.subject.clone(), subject_style));

    // Trailing metadata: date · author · hash
    spans.push(pad_span("  ", sel_bg));
    let meta = format!(
        "{}  {:<10}  {}",
        format_relative_date(row.timestamp),
        truncate_author(&row.author, 10),
        row.short_hash
    );
    let mut meta_style = Style::default().fg(Color::DarkGray);
    if let Some(bg) = sel_bg {
        meta_style = meta_style.bg(bg);
    }
    spans.push(Span::styled(meta, meta_style));

    Line::from(spans)
}

fn pad_span(text: &str, sel_bg: Option<Color>) -> Span<'static> {
    let mut style = Style::default();
    if let Some(bg) = sel_bg {
        style = style.bg(bg);
    }
    Span::styled(text.to_string(), style)
}

fn ref_pill(rf: &str, sel_bg: Option<Color>) -> Span<'static> {
    let (fg, label) = classify_ref(rf);
    let mut style = Style::default()
        .fg(fg)
        .add_modifier(Modifier::BOLD);
    if let Some(bg) = sel_bg {
        style = style.bg(bg);
    }
    Span::styled(format!("[{label}]"), style)
}

fn classify_ref(rf: &str) -> (Color, String) {
    let trimmed = rf.trim();
    if trimmed.starts_with("HEAD -> ") || trimmed == "HEAD" {
        (
            Color::Cyan,
            trimmed
                .strip_prefix("HEAD -> ")
                .map(|b| format!("HEAD→{b}"))
                .unwrap_or_else(|| trimmed.to_string()),
        )
    } else if trimmed.starts_with("tag: ") {
        (
            Color::Magenta,
            trimmed.to_string(),
        )
    } else if trimmed.contains('/') {
        // remotes / namespaced refs
        (Color::Yellow, trimmed.to_string())
    } else {
        (Color::Green, trimmed.to_string())
    }
}

fn lane_color(idx: u8) -> Color {
    const PALETTE: [Color; LANE_PALETTE_SIZE as usize] = [
        Color::Blue,
        Color::LightRed,
        Color::Cyan,
        Color::Yellow,
        Color::Magenta,
        Color::Green,
        Color::LightBlue,
        Color::LightYellow,
    ];
    PALETTE[(idx as usize) % PALETTE.len()]
}

fn format_relative_date(timestamp: i64) -> String {
    let Some(dt) = Utc.timestamp_opt(timestamp, 0).single() else {
        return "—".to_string();
    };
    let local = dt.with_timezone(&Local);
    let now = Local::now();
    let days = now.date_naive().signed_duration_since(local.date_naive()).num_days();
    if days == 0 {
        local.format("%H:%M").to_string()
    } else if days < 7 {
        local.format("%a %H:%M").to_string()
    } else {
        local.format("%d %b %Y").to_string()
    }
}

fn truncate_author(author: &str, max: usize) -> String {
    let mut out = String::new();
    for (i, ch) in author.chars().enumerate() {
        if i >= max {
            break;
        }
        out.push(ch);
    }
    out
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
                Style::default().add_modifier(Modifier::REVERSED)
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
                Style::default().add_modifier(Modifier::REVERSED)
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
