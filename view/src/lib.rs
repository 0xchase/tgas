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

// The input mode determines how key presses are handled.
enum InputMode {
    Normal, // Normal navigation
    Search, // Typing a search query
}

// The App struct holds the state of the application.
struct App {
    state: TableState,      // State for the table widget (e.g., selected row)
    df: DataFrame,          // The original, unfiltered DataFrame
    filtered_df: DataFrame, // The DataFrame after applying the search filter
    scroll_x: usize,        // Horizontal scroll position
    input_mode: InputMode,  // The current input mode
    input: String,          // The current value of the search input
}

impl App {
    fn new(lf: LazyFrame) -> Self {
        let df = lf.collect().unwrap_or_else(|_| DataFrame::default());
        let filtered_df = df.clone();
        let mut state = TableState::default();
        // Select the first row if the dataframe is not empty
        if !filtered_df.is_empty() {
            state.select(Some(0));
        }

        Self {
            state,
            df,
            filtered_df,
            scroll_x: 0,
            input_mode: InputMode::Normal,
            input: String::new(),
        }
    }

    // Move selection to the next row
    pub fn next(&mut self) {
        if self.filtered_df.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.filtered_df.height() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    // Move selection to the previous row
    pub fn previous(&mut self) {
        if self.filtered_df.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_df.height() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    // Scroll columns to the right
    pub fn next_col(&mut self) {
        if self.scroll_x < self.df.width().saturating_sub(1) {
            self.scroll_x += 1;
        }
    }

    // Scroll columns to the left
    pub fn previous_col(&mut self) {
        if self.scroll_x > 0 {
            self.scroll_x -= 1;
        }
    }

    // Apply the search filter to the DataFrame
    pub fn apply_filter(&mut self) {
        if self.input.is_empty() {
            self.filtered_df = self.df.clone();
        } else {
            // TODO: Implement proper string filtering
            // For now, just show all data
            self.filtered_df = self.df.clone();
            
            /* Original filtering code - needs fixing
            // Create a mask for each column and combine them
            let mut mask = BooleanChunked::full("mask", false, self.df.height());
            
            for col_name in self.df.get_column_names() {
                let col = self.df.column(col_name).unwrap();
                if let Ok(str_col) = col.cast(&DataType::String) {
                    if let Ok(str_col) = str_col.str() {
                        if let Ok(col_mask) = str_col.contains_literal(self.input.as_str()) {
                            mask = mask | col_mask;
                        }
                    }
                }
            }

            // Apply the filter
            self.filtered_df = self.df.filter(&mask).unwrap_or_else(|_| DataFrame::default());
            */
        }

        // Reset selection after filtering
        if !self.filtered_df.is_empty() {
            self.state.select(Some(0));
        } else {
            self.state.select(None);
        }
    }
}

// This function sets up the terminal and runs the main application loop.
pub fn run_tui(lf: LazyFrame) -> io::Result<()> {
    // --- Terminal setup ---
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(lf);
    let res = run_app(&mut terminal, &mut app);

    // --- Restore terminal ---
    // This code runs when the app exits, either normally or on error.
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

// This is the main application loop. It handles events and drawing.
fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('/') => app.input_mode = InputMode::Search,
                    KeyCode::Left => app.previous_col(),
                    KeyCode::Right => app.next_col(),
                    KeyCode::Down => app.next(),
                    KeyCode::Up => app.previous(),
                    _ => {}
                },
                InputMode::Search => match key.code {
                    KeyCode::Enter => {
                        app.apply_filter();
                        app.input_mode = InputMode::Normal;
                    }
                    KeyCode::Char(c) => app.input.push(c),
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                        app.input.clear();
                        app.apply_filter(); // Reset filter on escape
                    }
                    _ => {}
                },
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

    // Draw either the help message or the search box based on the current mode
    match app.input_mode {
        InputMode::Normal => {
            let help_text = "Use arrow keys to navigate rows/cols, '/' to search, 'q' to quit.";
            let help_message = Paragraph::new(help_text)
                .block(Block::default().borders(Borders::ALL).title("Help"));
            f.render_widget(help_message, chunks[1]);
        }
        InputMode::Search => {
            draw_search_box(f, app, chunks[1]);
        }
    }
}

// Draws the main data table.
fn draw_table(f: &mut Frame, app: &mut App, area: Rect) {
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default(); // Keep header simple
    let header_style = Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD);

    // Determine how many columns can fit in the visible area.
    let max_cols = (area.width / 20).max(1) as usize;

    let header_cells: Vec<Cell> = app
        .filtered_df
        .get_column_names()
        .iter()
        .skip(app.scroll_x)
        .take(max_cols)
        .map(|h| Cell::from(h.to_string()).style(header_style))
        .collect();

    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);

    // Create rows from the DataFrame data
    let rows: Vec<Row> = app
        .filtered_df
        .iter()
        .skip(app.state.offset()) // Use table state offset for scrolling
        .take(area.height as usize) // Take enough to fill the view
        .map(|row_tuple| {
            let cells: Vec<Cell> = row_tuple
                .iter()
                .skip(app.scroll_x)
                .take(max_cols)
                .map(|val| Cell::from(val.to_string()))
                .collect();
            Row::new(cells).height(1)
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

// Draws the search input box.
fn draw_search_box(f: &mut Frame, app: &App, area: Rect) {
    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Search"));
    f.render_widget(input, area);
    // Show the cursor at the end of the input
    f.set_cursor(area.x + app.input.len() as u16 + 1, area.y + 1);
}
