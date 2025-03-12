use color_eyre::Result;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::r2d2::PooledConnection;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::Event;
use ratatui::crossterm::event::KeyCode;
use ratatui::crossterm::event::KeyEventKind;
use ratatui::crossterm::event::{self};

use nanuak_schema::youtube::videos;

use crate::db::SearchResult;
use crate::db::get_all_results;
use crate::durations::parse_duration_str;
use crate::ui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveInputField {
    SearchTerm,
    MinDuration,
    MaxDuration,
}

pub struct App {
    pub search_term: String,
    pub min_duration: String,
    pub max_duration: String,
    pub active_field: ActiveInputField,
    pub results: Vec<SearchResult>,
    pub results_state: ratatui::widgets::ListState,
    #[allow(unused)]
    pub pool: Pool<ConnectionManager<PgConnection>>,
}

impl App {
    pub async fn new(
        pool: Pool<ConnectionManager<PgConnection>>,
        conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    ) -> Result<Self> {
        let results = get_all_results(conn).await?;
        let mut results_state = ratatui::widgets::ListState::default();
        results_state.select(Some(0));

        Ok(Self {
            search_term: String::new(),
            min_duration: String::new(),
            max_duration: String::new(),
            active_field: ActiveInputField::SearchTerm,
            results,
            results_state,
            pool,
        })
    }


    pub async fn update_search_results(&mut self, conn: &mut PgConnection) -> Result<()> {
        let pattern = format!("%{}%", self.search_term);
        // Start with a base query
        let mut query = videos::table
            .select((videos::title, videos::video_id, videos::duration))
            .filter(videos::title.ilike(&pattern))
            .into_boxed();

        // If the "videos" table has a "duration" column in seconds, we can filter as below:
        if let Some(min_secs) = parse_duration_str(&self.min_duration) {
            query = query.filter(videos::duration.ge(min_secs));
        }
        if let Some(max_secs) = parse_duration_str(&self.max_duration) {
            query = query.filter(videos::duration.le(max_secs));
        }

        let new_results = query.load::<SearchResult>(conn)?;
        self.results = new_results;

        // Reset the selected index
        if self.results.is_empty() {
            self.results_state.select(None);
        } else {
            self.results_state.select(Some(0));
        }

        Ok(())
    }

    pub async fn run(
        mut self,
        terminal: &mut DefaultTerminal,
        conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    ) -> eyre::Result<()> {
        loop {
            terminal.draw(|f| ui::draw(f, &mut self))?;
            let Event::Key(key) = event::read()? else {
                continue;
            };
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Esc => break,

                // Tab to cycle which field is active
                KeyCode::Tab => {
                    self.active_field = match self.active_field {
                        ActiveInputField::SearchTerm => ActiveInputField::MinDuration,
                        ActiveInputField::MinDuration => ActiveInputField::MaxDuration,
                        ActiveInputField::MaxDuration => ActiveInputField::SearchTerm,
                    };
                }

                KeyCode::Backspace => {
                    if key.kind == KeyEventKind::Press {
                        let term = match self.active_field {
                            ActiveInputField::SearchTerm => &mut self.search_term,
                            ActiveInputField::MinDuration => &mut self.min_duration,
                            ActiveInputField::MaxDuration => &mut self.max_duration,
                        };
                        if !term.is_empty() {
                            term.pop();
                        }
                        self.update_search_results(conn).await?;
                    }
                }

                KeyCode::Char(c) => {
                    if key.kind == KeyEventKind::Press {
                        // If user pressed SHIFT+Tab or SHIFT+something, interpret as uppercase if needed
                        // but usually KeyEventKind + KeyModifiers helps.
                        // We'll keep it simple and always push the char 'c':
                        let term = match self.active_field {
                            ActiveInputField::SearchTerm => &mut self.search_term,
                            ActiveInputField::MinDuration => &mut self.min_duration,
                            ActiveInputField::MaxDuration => &mut self.max_duration,
                        };
                        term.push(c);
                        self.update_search_results(conn).await?;
                    }
                }

                KeyCode::Down => {
                    ui::select_next(&mut self.results_state, self.results.len());
                }

                KeyCode::Up => {
                    ui::select_previous(&mut self.results_state, self.results.len());
                }

                _ => {}
            }
        }

        Ok(())
    }
}
