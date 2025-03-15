use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{BarChart, Block, Borders},
    Frame, Terminal,
};
use std::{error::Error, io};

struct App {
    should_quit: bool,
}

impl App {
    fn new() -> App {
        App { should_quit: false }
    }

    fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            _ => {}
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            app.on_key(key.code);
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn ui(f: &mut Frame, _app: &App) {
    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.area());

    // Bar chart data
    let data = [("B1", 45), ("B2", 72), ("B3", 38)];

    // Create bar chart
    let bar_chart = BarChart::default()
        .block(
            Block::default()
                .title("Bar Chart Example")
                .borders(Borders::ALL),
        )
        .data(&data)
        .bar_width(9)
        .bar_gap(3)
        .bar_style(Style::default().fg(Color::LightBlue))
        .value_style(Style::default().fg(Color::White))
        .label_style(Style::default().fg(Color::Yellow));

    f.render_widget(bar_chart, chunks[0]);
}
