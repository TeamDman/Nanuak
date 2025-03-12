use color_eyre::Result;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use nanuak_config::config::NanuakConfig;
use nanuak_config::db_url::DatabasePassword;
use nanuak_schema::youtube::videos;
use r2d2::PooledConnection;
use ratatui::DefaultTerminal;
use ratatui::Frame;
use ratatui::crossterm::event::Event;
use ratatui::crossterm::event::KeyCode;
use ratatui::crossterm::event::KeyEventKind;
use ratatui::crossterm::event::{self};
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
use ratatui::widgets::Paragraph;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let database_url = DatabasePassword::format_url(
        &NanuakConfig::acquire()
            .await?
            .get::<DatabasePassword>()
            .await?,
    );

    // Set up a database connection pool
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder().build(manager)?;
    let mut conn = pool.get()?;
    info!("Established database connection");
    let mut terminal = ratatui::init();
    let app = App::new(pool, &mut conn).await?;
    let result = app.run(&mut terminal, &mut conn).await;
    ratatui::restore();
    result
}

struct App {
    search_term: String,
    results: Vec<SearchResult>,
    results_state: ratatui::widgets::ListState,
    pool: Pool<ConnectionManager<PgConnection>>,
}

#[derive(Debug, Queryable)]
struct SearchResult {
    title: String,
    video_id: String,
}

impl App {
    async fn new(
        pool: Pool<ConnectionManager<PgConnection>>,
        conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    ) -> Result<Self> {
        let results = get_all_results(conn).await?;
        let mut results_state = ratatui::widgets::ListState::default();
        results_state.select(Some(0));
        Ok(App {
            search_term: String::new(),
            results,
            results_state,
            pool,
        })
    }

    fn draw(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(f.area());

        let search_block = Paragraph::new(format!("Search: {}", self.search_term))
            .block(Block::default().borders(Borders::ALL).title("Search"));
        f.render_widget(search_block, chunks[0]);

        let results_block = Block::default().borders(Borders::ALL).title("Results");
        let items: Vec<ListItem> = self
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

        f.render_stateful_widget(results_list, chunks[1], &mut self.results_state);
    }

    async fn update_search_results(&mut self, conn: &mut PgConnection) -> Result<()> {
        let pattern = format!("%{}%", self.search_term);

        let new_results = videos::table
            .select((videos::title, videos::video_id))
            .filter(videos::title.ilike(pattern))
            .load::<SearchResult>(conn)?;

        self.results = new_results;
        // Reset the selected index to 0 if results are not empty, otherwise select None.
        self.results_state.select(Some(0));
        Ok(())
    }

    async fn run(
        self,
        terminal: &mut DefaultTerminal,
        conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    ) -> eyre::Result<()> {
        let mut app = self;
        loop {
            terminal.draw(|f| app.draw(f))?;
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => break,
                    KeyCode::Char(c) => {
                        if key.kind == KeyEventKind::Press {
                            app.search_term.push(c);
                            app.update_search_results(conn).await?;
                        }
                    }
                    KeyCode::Backspace => {
                        if key.kind == KeyEventKind::Press {
                            app.search_term.pop();
                            app.update_search_results(conn).await?;
                        }
                    }
                    KeyCode::Down => app.results_state.select_next(),
                    KeyCode::Up => app.results_state.select_previous(),
                    _ => {}
                }
            }
        }
        Ok(())
    }
}

async fn get_all_results(conn: &mut PgConnection) -> Result<Vec<SearchResult>> {
    videos::table
        .select((videos::title, videos::video_id))
        .load::<SearchResult>(conn)
        .map_err(Into::into)
}
