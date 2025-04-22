use cli::figma_conversion::Project;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, List, ListItem, Tabs},
};
use std::io;

// Application state structure
struct App {
    current_tab: usize,
    tabs: Vec<&'static str>,
    projects: Vec<Project>,
    selected_project: Option<usize>,
    conversion_progress: f64,
    logs: Vec<String>,
}

impl App {
    fn new() -> Self {
        Self {
            current_tab: 0,
            tabs: vec!["Projects", "Conversion", "Logs", "Settings"],
            projects: vec![/* loaded from config */],
            selected_project: None,
            conversion_progress: 0.0,
            logs: vec![],
        }
    }

    // Update progress during conversion
    fn update_progress(&mut self, progress: f64) {
        self.conversion_progress = progress;
        self.logs
            .push(format!("Progress updated: {:.1}%", progress * 100.0));
    }

    // Add a log message
    fn log(&mut self, message: String) {
        self.logs.push(message);
    }
}

fn main() -> Result<(), io::Error> {
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();

    // Main event loop
    loop {
        terminal.draw(|f| {
            // UI rendering logic
            let size = f.size();

            // Layout with tabs at top, main content area below
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(size);

            // Render tabs
            let tabs = Tabs::new(app.tabs.iter().map(|t| t.to_string()).collect())
                .select(app.current_tab)
                .block(Block::default().borders(Borders::ALL).title("Etch CLI"))
                .style(Style::default().fg(Color::White))
                .highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                );
            f.render_widget(tabs, chunks[0]);

            // Render main content based on selected tab
            match app.current_tab {
                0 => {
                    // Projects list
                }
                1 => {
                    // Conversion progress
                    let progress_layout = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                        .split(chunks[1]);

                    let gauge = Gauge::default()
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title("Conversion Progress"),
                        )
                        .gauge_style(Style::default().fg(Color::Green))
                        .percent((app.conversion_progress * 100.0) as u16);

                    f.render_widget(gauge, progress_layout[0]);
                }
                2 => {
                    // Logs view
                    let logs: Vec<ListItem> =
                        app.logs.iter().map(|l| ListItem::new(l.clone())).collect();

                    let logs_list = List::new(logs)
                        .block(Block::default().borders(Borders::ALL).title("Logs"))
                        .style(Style::default().fg(Color::White));

                    f.render_widget(logs_list, chunks[1]);
                }
                _ => {
                    // Settings
                    // ...
                }
            }
        })?;

        // Handle events
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Tab => {
                    app.current_tab = (app.current_tab + 1) % app.tabs.len();
                }
                KeyCode::BackTab => {
                    app.current_tab = (app.current_tab + app.tabs.len() - 1) % app.tabs.len();
                }
                // Add more key handling...
                _ => {}
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}
