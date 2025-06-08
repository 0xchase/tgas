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
        if !df.is_empty() {
            state.select(Some(0));
        }

        Self {
            state,
            df,
            scroll_x: 0,
            viewport_height: 0, // Will be updated on first render
        }
    }

    // Move selection to the next row, halting at the end.
    pub fn next(&mut self) {
        if self.df.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            // Halt at the last item
            Some(i) => (i + 1).min(self.df.height().saturating_sub(1)),
            None => 0,
        };
        self.state.select(Some(i));

        // --- Manual "Sliding Window" Scroll Logic ---
        let row_count = self.viewport_height.saturating_sub(3);
        if row_count > 0 {
            let offset = self.state.offset();
            // If the selection is at or below the bottom of the viewport, scroll down.
            if i >= offset + row_count {
                *self.state.offset_mut() = offset + 1;
            }
        }
    }

    // Move selection to the previous row, halting at the beginning.
    pub fn previous(&mut self) {
        if self.df.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            // Halt at the first item
            Some(i) => i.saturating_sub(1),
            None => 0,
        };
        self.state.select(Some(i));

        let offset = self.state.offset();
        // If selection moves above the viewport, scroll up.
        if i < offset {
            *self.state.offset_mut() = offset - 1;
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

    let header = Row::new(header_cells).height(1);

    // --- Lazy Row Rendering ---
    let start_row = app.state.offset();
    let row_count = app.viewport_height.saturating_sub(3);
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

    // --- Render with a relative selection and zero offset to fix the double-slice bug ---
    // Stash the absolute selection and offset.
    let abs_sel = app.state.selected();
    let abs_offset = app.state.offset();

    // Compute a relative index for the visible window.
    let rel_sel = abs_sel.and_then(|s| {
        if s >= start_row && s < end_row {
            Some(s - start_row)
        } else {
            None
        }
    });

    // Temporarily modify state for rendering: set relative selection and zero offset.
    app.state.select(rel_sel);
    *app.state.offset_mut() = 0;

    // Render the widget.
    f.render_stateful_widget(table, area, &mut app.state);

    // Restore the original absolute state for the app's own logic.
    app.state.select(abs_sel);
    *app.state.offset_mut() = abs_offset;
}
