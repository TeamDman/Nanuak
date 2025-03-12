use color_eyre::Result;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use eyre::WrapErr;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;

use nanuak_schema::youtube::videos;

use crate::db::{get_all_results, SearchResult};
use crate::ui;

pub struct App {
    pub search_term: String,
    pub results: Vec<SearchResult>,
    pub results_state: ratatui::widgets::ListState,
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
            results,
            results_state,
            pool,
        })
    }

    pub async fn update_search_results(&mut self, conn: &mut PgConnection) -> Result<()> {
        let pattern = format!("%{}%", self.search_term);

        let new_results = videos::table
            .select((videos::title, videos::video_id))
            .filter(videos::title.ilike(pattern))
            .load::<SearchResult>(conn)?;

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

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => break,

                    KeyCode::Char(c) => {
                        if key.kind == KeyEventKind::Press {
                            self.search_term.push(c);
                            self.update_search_results(conn).await?;
                        }
                    }

                    KeyCode::Backspace => {
                        if key.kind == KeyEventKind::Press && !self.search_term.is_empty() {
                            self.search_term.pop();
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
        }

        Ok(())
    }
}
