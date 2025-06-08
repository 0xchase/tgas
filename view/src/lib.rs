use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use polars::prelude::*;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame, Terminal,
};
use std::io;

// The App struct holds the state of the application.
struct App {
    state: TableState,      // State for the table widget (e.g., selected row and offset)
    df: DataFrame,          // The DataFrame being displayed
    scroll_x: usize,        // Horizontal scroll position
    viewport_height: usize, // The number of rows visible in the table area
}

impl App {
    fn new(lf: LazyFrame) -> Self {
        let df = lf.collect().unwrap_or_else(|_| DataFrame::default());
        let mut state = TableState::default();
        // Select the first row if the dataframe is not empty
        if !df.is_empty() {
            state.select(Some(0));
        }

        Self {
            state,
            df,
            scroll_x: 0,
            viewport_height: 0, // Initialized to 0, will be updated on first render
        }
    }

    // Move selection to the next row, wrapping around at the end.
    pub fn next(&mut self) {
        if self.df.is_empty() {
            return;
        }
        let i = self.state.selected().map_or(0, |i| {
            if i >= self.df.height() - 1 {
                0
            } else {
                i + 1
            }
        });
        self.state.select(Some(i));

        // --- Manual Scroll Logic ---
        // If the new selection is outside the viewport, adjust the scroll offset.
        let viewport_height = self.viewport_height.saturating_sub(3); // Account for borders/header
        if viewport_height > 0 {
            let offset = self.state.offset();
            if i >= offset + viewport_height {
                // If selection is below the view, set offset to bring it into view from the bottom.
                *self.state.offset_mut() = i - viewport_height + 1;
            } else if i < offset {
                 // This case handles wrapping around from the end to the beginning.
                *self.state.offset_mut() = i;
            }
        }
    }

    // Move selection to the previous row, wrapping around at the beginning.
    pub fn previous(&mut self) {
        if self.df.is_empty() {
            return;
        }
        let i = self.state.selected().map_or(0, |i| {
            if i == 0 {
                self.df.height() - 1
            } else {
                i - 1
            }
        });
        self.state.select(Some(i));

        // --- Manual Scroll Logic ---
        // If the new selection is outside the viewport, adjust the scroll offset.
        let offset = self.state.offset();
        if i < offset {
            // If selection is above the view, set offset to bring it into view from the top.
            *self.state.offset_mut() = i;
        } else if i >= offset + self.viewport_height.saturating_sub(3) {
            // This case handles wrapping around from the beginning to the end.
            let viewport_height = self.viewport_height.saturating_sub(3);
            if viewport_height > 0 {
                *self.state.offset_mut() = i - viewport_height + 1;
            }
        }
    }

    // Scroll columns to the right.
    pub fn next_col(&mut self) {
        self.scroll_x = self.scroll_x.saturating_add(1).min(self.df.width().saturating_sub(1));
    }

    // Scroll columns to the left.
    pub fn previous_col(&mut self) {
        self.scroll_x = self.scroll_x.saturating_sub(1);
    }
}

// This function sets up the terminal and runs the main application loop.
pub fn run_tui(lf: LazyFrame) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(lf);
    let res = run_app(&mut terminal, &mut app);

    // This code ensures the terminal is restored to its original state.
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

// This is the main application loop. It handles events and drawing.
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

// This function draws the main UI widgets.
fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
        .split(f.size());

    draw_table(f, app, chunks[0]);

    // Display a simple help message.
    let help_text = "Use arrow keys to navigate rows/cols, 'q' to quit.";
    let help_message =
        Paragraph::new(help_text).block(Block::default().borders(Borders::ALL).title("Help"));
    f.render_widget(help_message, chunks[1]);
}

// Draws the main data table.
fn draw_table(f: &mut Frame, app: &mut App, area: Rect) {
    // Update the app's viewport height. This is used by the navigation logic.
    app.viewport_height = area.height as usize;

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let header_style = Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD);

    let max_cols = (area.width / 20).max(1) as usize;

    let header_cells: Vec<Cell> = app
        .df
        .get_column_names()
        .iter()
        .skip(app.scroll_x)
        .take(max_cols)
        .map(|h| Cell::from(h.to_string()).style(header_style))
        .collect();

    let header = Row::new(header_cells).height(1).bottom_margin(1);

    // --- Lazy Row Generation ---
    // We only create Rows for the visible part of the DataFrame.
    let start_row = app.state.offset();
    let row_count = app.viewport_height.saturating_sub(3); // space for header/borders
    let end_row = (start_row + row_count).min(app.df.height());

    let rows: Vec<ratatui::widgets::Row> = (start_row..end_row)
        .map(|i| {
            let polars_row = app.df.get_row(i).unwrap();
            let cells: Vec<Cell> = polars_row.0
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
        .block(Block::default().borders(Borders::ALL).title("Polars DataFrame Explorer"))
        .highlight_style(selected_style)
        .highlight_symbol(">> ");

    f.render_stateful_widget(table, area, &mut app.state);
}
