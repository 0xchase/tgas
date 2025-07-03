use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use polars::prelude::*;
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
};
use std::io;

struct App {
    state: TableState,
    df: DataFrame,
    scroll_x: usize,
    viewport_height: usize,
}

impl App {
    fn new(lf: LazyFrame) -> Self {
        let df = lf.collect().unwrap_or_else(|_| DataFrame::default());
        let mut state = TableState::default();
        if !df.is_empty() {
            state.select(Some(0));
        }

        Self {
            state,
            df,
            scroll_x: 0,
            viewport_height: 0,
        }
    }

    pub fn next(&mut self) {
        if self.df.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => (i + 1).min(self.df.height().saturating_sub(1)),
            None => 0,
        };
        self.state.select(Some(i));

        let row_count = self.viewport_height.saturating_sub(3);
        if row_count > 0 {
            let offset = self.state.offset();
            if i >= offset + row_count {
                *self.state.offset_mut() = offset + 1;
            }
        }
    }

    pub fn previous(&mut self) {
        if self.df.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => i.saturating_sub(1),
            None => 0,
        };
        self.state.select(Some(i));

        let offset = self.state.offset();
        if i < offset {
            *self.state.offset_mut() = offset - 1;
        }
    }

    pub fn next_col(&mut self) {
        self.scroll_x = self
            .scroll_x
            .saturating_add(1)
            .min(self.df.width().saturating_sub(1));
    }

    pub fn previous_col(&mut self) {
        self.scroll_x = self.scroll_x.saturating_sub(1);
    }
}

pub fn run_tui(lf: LazyFrame) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(lf);
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Left => app.previous_col(),
                KeyCode::Right => app.next_col(),
                KeyCode::Down => app.next(),
                KeyCode::Up => app.previous(),
                _ => {}
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
        .split(f.size());

    draw_table(f, app, chunks[0]);

    let help_text = "Use arrow keys to navigate rows/cols, 'q' to quit.";
    let help_message =
        Paragraph::new(help_text).block(Block::default().borders(Borders::ALL).title("Help"));
    f.render_widget(help_message, chunks[1]);
}

fn draw_table(f: &mut Frame, app: &mut App, area: Rect) {
    app.viewport_height = area.height as usize;

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let header_style = Style::default()
        .fg(Color::Blue)
        .add_modifier(Modifier::BOLD);

    let max_cols = (area.width / 20).max(1) as usize;

    let header_cells: Vec<Cell> = app
        .df
        .get_column_names()
        .iter()
        .skip(app.scroll_x)
        .take(max_cols)
        .map(|h| Cell::from(h.to_string()).style(header_style))
        .collect();

    let header = Row::new(header_cells).height(1);

    let start_row = app.state.offset();
    let row_count = app.viewport_height.saturating_sub(3);
    let end_row = (start_row + row_count).min(app.df.height());

    let rows: Vec<ratatui::widgets::Row> = (start_row..end_row)
        .map(|i| {
            let polars_row = app.df.get_row(i).unwrap();
            let cells: Vec<Cell> = polars_row
                .0
                .iter()
                .skip(app.scroll_x)
                .take(max_cols)
                .map(|val| Cell::from(val.to_string()))
                .collect();
            ratatui::widgets::Row::new(cells).height(1)
        })
        .collect();

    let widths = (0..max_cols)
        .map(|_| Constraint::Length(20))
        .collect::<Vec<_>>();

    let table = Table::new(rows, &widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Polars DataFrame Explorer"),
        )
        .highlight_style(selected_style)
        .highlight_symbol(">> ");

    let abs_sel = app.state.selected();
    let abs_offset = app.state.offset();

    let rel_sel = abs_sel.and_then(|s| {
        if s >= start_row && s < end_row {
            Some(s - start_row)
        } else {
            None
        }
    });

    app.state.select(rel_sel);
    *app.state.offset_mut() = 0;

    f.render_stateful_widget(table, area, &mut app.state);

    app.state.select(abs_sel);
    *app.state.offset_mut() = abs_offset;
}
