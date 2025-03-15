use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::Rng;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::Rect,
    style::{Color, Style},
    widgets::{Bar, BarChart, BarGroup, Block, Borders},
    Frame, Terminal,
};
use std::{error::Error, io, time::Duration};

struct App {
    data: Vec<(String, u64)>,
    should_quit: bool,
}

impl App {
    fn new(data: Vec<(String, u64)>) -> App {
        App {
            data,
            should_quit: false,
        }
    }

    fn modify_data(data: &mut Vec<(String, u64)>) {
        let mut rng = rand::rng();

        for (_, value) in data.iter_mut() {
            let change = rng.random_range(-1..=1);
            *value = value.saturating_add_signed(change);
        }
    }

    fn tick(&mut self) {
        Self::modify_data(&mut self.data);
        // println!("modified data: {:?}", self.data);
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
    let mut app = App::new(
        [("B1", 45), ("B2", 72), ("B3", 38)]
            .into_iter()
            .map(|(name, val)| (name.to_string(), val))
            .collect(),
    );
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
        app.tick();
        terminal.draw(|f| ui(f, app))?;
        if crossterm::event::poll(Duration::from_secs(1))? {
            if let Event::Key(key) = event::read()? {
                app.on_key(key.code);
            }

            if app.should_quit {
                return Ok(());
            }
        }
    }
}

fn bar_group_from_app(app: &App) -> BarGroup {
    BarGroup::from(
        &app.data
            .iter()
            .map(|datum| (datum.0.as_str(), datum.1))
            .collect::<Vec<(&str, u64)>>(),
    )
}

fn ui(f: &mut Frame, app: &App) {
    // Calculate the width needed for the chart
    // For each bar: width + gap = 9 + 3 = 12 units
    // Last bar doesn't need a gap, plus add some padding and borders
    let bar_width = 9;
    let bar_gap = 3;
    let num_bars = app.data.len();
    let total_width = (bar_width + bar_gap) * (num_bars - 1) + bar_width + 2; // +2 for borders.

    // Create a centered area with just enough width for our bars
    let area = centered_rect(total_width as u16, 20, f.area());

    // Create bar chart
    let bar_chart = BarChart::default()
        .block(
            Block::default()
                .title("Bar Chart Example")
                .borders(Borders::ALL),
        )
        .data(bar_group_from_app(app))
        .bar_width(bar_width as u16)
        .bar_gap(bar_gap as u16)
        .bar_style(Style::default().fg(Color::LightBlue))
        .value_style(Style::default().fg(Color::White))
        .label_style(Style::default().fg(Color::Yellow));

    f.render_widget(bar_chart, area);
}

// Helper function to create a centered rect using fixed width/height
fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let x = (r.width.saturating_sub(width)) / 2;
    let y = (r.height.saturating_sub(height)) / 2;

    Rect {
        x: r.x + x,
        y: r.y + y,
        width: width.min(r.width),
        height: height.min(r.height),
    }
}
