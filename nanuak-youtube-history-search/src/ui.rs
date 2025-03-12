use std::time::Duration;

use crate::app::ActiveInputField;
use crate::app::App;
use crate::durations::format_duration;
use chrono::Local;
use chrono::NaiveDateTime;
use chrono::Utc;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::List;
use ratatui::widgets::ListItem;
use ratatui::widgets::ListState;
use ratatui::widgets::Paragraph;

fn format_ago_opt(last_watch: Option<NaiveDateTime>) -> String {
    if let Some(ts) = last_watch {
        let now = Utc::now().naive_utc();
        let diff = now.signed_duration_since(ts);
        if diff.num_seconds() < 0 {
            return "time anomaly?".to_string();
        }
        format!("{} ago", format_duration(&diff))
    } else {
        "never watched".to_string()
    }
}

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(5), Constraint::Min(0)].as_ref())
        .split(f.area());

    let normal_style = Style::default().fg(Color::White);
    let active_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);

    let search_span = if app.active_field == ActiveInputField::SearchTerm {
        Span::styled(format!("{}_", app.search_term), active_style)
    } else {
        Span::styled(app.search_term.clone(), normal_style)
    };
    let min_span = if app.active_field == ActiveInputField::MinDuration {
        Span::styled(format!("{}_", app.min_duration), active_style)
    } else {
        Span::styled(app.min_duration.clone(), normal_style)
    };
    let max_span = if app.active_field == ActiveInputField::MaxDuration {
        Span::styled(format!("{}_", app.max_duration), active_style)
    } else {
        Span::styled(app.max_duration.clone(), normal_style)
    };
    let ago_span = if app.active_field == ActiveInputField::Ago {
        Span::styled(format!("{}_", app.ago), active_style)
    } else {
        Span::styled(app.ago.clone(), normal_style)
    };

    let search_line = Line::from(vec![
        Span::raw("Search: "),
        search_span,
        Span::raw(" | MinDur: "),
        min_span,
        Span::raw(" | MaxDur: "),
        max_span,
        Span::raw(" | Ago: "),
        ago_span,
    ]);

    let refresh_line = match app.refresh_at {
        Some(refresh_at) => {
            let now = Local::now();
            let diff = refresh_at.signed_duration_since(now).num_milliseconds();
            if diff < 0 {
                Line::from("")
            } else {
                let diff = Duration::from_millis(diff as u64);
                let duration_display = humantime::format_duration(diff).to_string();
                let refresh_line = Line::from(vec![
                    Span::raw("Refreshing in: "),
                    Span::styled(duration_display, Style::default().fg(Color::Red)),
                ]);
                refresh_line
            }
        }
        None => Line::from(""),
    };

    let filters_paragraph = Paragraph::new(vec![search_line, refresh_line])
        .block(Block::default().borders(Borders::ALL).title("Filters"));
    f.render_widget(filters_paragraph, chunks[0]);

    let results_block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Results - {}", app.results.0.len()));
    let results_list: List = match app.results.1.take() {
        None => {
            let items: Vec<ListItem> = app
                .results
                .0
                .iter()
                .map(|res| {
                    let ago_str = format_ago_opt(res.last_watch);
                    let dur_str = format_duration(&res.duration);

                    // ex: "Title (link) [4m21s], 1w6d ago"
                    let line = Line::from(vec![
                        Span::styled(
                            res.title.clone(),
                            Style::default()
                                .fg(Color::LightBlue)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(" "),
                        Span::styled(
                            format!("(https://youtube.com/watch?v={})", res.video_id),
                            Style::default().fg(Color::Gray),
                        ),
                        Span::styled(format!(" [{}]", dur_str), Style::default().fg(Color::Green)),
                        Span::raw(", "),
                        Span::styled(ago_str, Style::default().fg(Color::Magenta)),
                    ]);
                    ListItem::new(line)
                })
                .collect();
            let results_list = List::new(items)
                .block(results_block)
                .highlight_style(Style::default().add_modifier(Modifier::BOLD));
            results_list
        }
        Some(items) => items,
    };

    f.render_stateful_widget(&results_list, chunks[1], &mut app.results_state);
    app.results.1 = Some(results_list);
}

pub fn select_next(state: &mut ListState, len: usize) {
    if len == 0 {
        return;
    }
    let i = match state.selected() {
        Some(i) => {
            if i >= len - 1 {
                0
            } else {
                i + 1
            }
        }
        None => 0,
    };
    state.select(Some(i));
}

pub fn select_previous(state: &mut ListState, len: usize) {
    if len == 0 {
        return;
    }
    let i = match state.selected() {
        Some(i) => {
            if i == 0 {
                len - 1
            } else {
                i - 1
            }
        }
        None => 0,
    };
    state.select(Some(i));
}
