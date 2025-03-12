use std::time::Duration;

use crate::db::SearchResult;
use crate::db::get_filtered_results;
use crate::durations::parse_duration_str;
use crate::ui;
use chrono::DateTime;
use chrono::Local;
use color_eyre::Result;
use diesel::PgConnection;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::r2d2::PooledConnection;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::Event;
use ratatui::crossterm::event::KeyCode;
use ratatui::crossterm::event::KeyEventKind;
use ratatui::crossterm::event::KeyModifiers;
use ratatui::crossterm::event::{self};
use ratatui::widgets::List;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveInputField {
    SearchTerm,
    MinDuration,
    MaxDuration,
    Ago,
}

pub struct App<'a> {
    pub search_term: String,
    pub min_duration: String,
    pub max_duration: String,
    pub ago: String,

    pub active_field: ActiveInputField,
    pub results: (Vec<SearchResult>, Option<List<'a>>),
    pub results_state: ratatui::widgets::ListState,
    pub refresh_at: Option<DateTime<Local>>,

    #[allow(unused)]
    pub pool: Pool<ConnectionManager<PgConnection>>,
}

impl<'a> App<'a> {
    pub async fn new(
        pool: Pool<ConnectionManager<PgConnection>>,
        conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    ) -> Result<Self> {
        let results = get_filtered_results(conn, "%".to_string(), None, None, None).await?;

        let mut results_state = ratatui::widgets::ListState::default();
        if !results.is_empty() {
            results_state.select(Some(0));
        }

        Ok(Self {
            search_term: String::new(),
            min_duration: String::new(),
            max_duration: String::new(),
            ago: String::new(),
            active_field: ActiveInputField::SearchTerm,
            results: (results, None),
            refresh_at: None,
            results_state,
            pool,
        })
    }

    pub async fn update_search_results(&mut self, conn: &mut PgConnection) -> Result<()> {
        let pattern = format!("%{}%", self.search_term);

        let min_secs = parse_duration_str(&self.min_duration).map(|d| d.num_seconds());
        let max_secs = parse_duration_str(&self.max_duration).map(|d| d.num_seconds());
        let ago_secs = parse_duration_str(&self.ago).map(|d| d.num_seconds());

        let new_results = get_filtered_results(conn, pattern, min_secs, max_secs, ago_secs).await?;

        self.results = (new_results, None);
        if self.results.0.is_empty() {
            self.results_state.select(None);
        } else {
            self.results_state.select(Some(0));
        }

        Ok(())
    }

    pub fn queue_refresh(&mut self) {
        let refresh_at = Local::now() + chrono::Duration::milliseconds(300);
        self.refresh_at = Some(refresh_at);
    }

    pub fn get_active_field(&mut self) -> &mut String {
        match self.active_field {
            ActiveInputField::SearchTerm => &mut self.search_term,
            ActiveInputField::MinDuration => &mut self.min_duration,
            ActiveInputField::MaxDuration => &mut self.max_duration,
            ActiveInputField::Ago => &mut self.ago,
        }
    }

    pub async fn run(
        mut self,
        terminal: &mut DefaultTerminal,
        conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    ) -> Result<()> {
        loop {
            if let Some(when) = &self.refresh_at {
                if Local::now() > *when {
                    self.update_search_results(conn).await?;
                    self.refresh_at = None;
                }
            }

            terminal.draw(|f| ui::draw(f, &mut self))?;

            // Check if input available
            if !event::poll(Duration::from_millis(16))? {
                continue;
            }

            // Grab the input
            let Event::Key(key) = event::read()? else {
                continue;
            };

            // Only care about presses
            if key.kind != KeyEventKind::Press {
                continue;
            };
            match key.code {
                KeyCode::Esc => break,

                KeyCode::Tab => {
                    self.active_field = match self.active_field {
                        ActiveInputField::SearchTerm => ActiveInputField::MinDuration,
                        ActiveInputField::MinDuration => ActiveInputField::MaxDuration,
                        ActiveInputField::MaxDuration => ActiveInputField::Ago,
                        ActiveInputField::Ago => ActiveInputField::SearchTerm,
                    };
                }

                x if matches!(x, KeyCode::Char('w') | KeyCode::Backspace)
                    && key.modifiers.contains(KeyModifiers::CONTROL) =>
                {
                    let field = self.get_active_field();
                    if !field.is_empty() {
                        field.clear();
                        self.queue_refresh();
                    }
                }
                KeyCode::Backspace => {
                    let field = self.get_active_field();
                    if !field.is_empty() {
                        field.pop();
                        self.queue_refresh();
                    }
                }

                KeyCode::Char(c) => {
                    let s = match self.active_field {
                        ActiveInputField::SearchTerm => &mut self.search_term,
                        ActiveInputField::MinDuration => &mut self.min_duration,
                        ActiveInputField::MaxDuration => &mut self.max_duration,
                        ActiveInputField::Ago => &mut self.ago,
                    };
                    s.push(c);
                    self.queue_refresh();
                }

                KeyCode::Down => {
                    ui::select_next(&mut self.results_state, self.results.0.len());
                }

                KeyCode::Up => {
                    ui::select_previous(&mut self.results_state, self.results.0.len());
                }

                _ => {}
            }
        }
        Ok(())
    }
}
