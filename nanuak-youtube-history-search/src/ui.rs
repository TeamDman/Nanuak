use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::app::App;

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.area());

    // Search box
    let search_block = Paragraph::new(format!("Search: {}", app.search_term))
        .block(Block::default().borders(Borders::ALL).title("Search"));
    f.render_widget(search_block, chunks[0]);

    // Results
    let results_block = Block::default().borders(Borders::ALL).title("Results");
    let items: Vec<ListItem> = app
        .results
        .iter()
        .map(|result| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    &result.title,
                    Style::default()
                        .fg(Color::LightBlue)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("(https://youtube.com/watch?v={})", result.video_id),
                    Style::default().fg(Color::Gray),
                ),
            ]))
        })
        .collect();

    let results_list = List::new(items)
        .block(results_block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    f.render_stateful_widget(results_list, chunks[1], &mut app.results_state);
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
