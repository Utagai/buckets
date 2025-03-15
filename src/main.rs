use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::{FutureExt, StreamExt};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::Rect,
    style::{Color, Style},
    widgets::{BarChart, Block, Borders},
    Frame, Terminal,
};
use std::{
    error::Error,
    io::{self, Stdout},
    sync::Arc,
    time::Duration,
};
use tokio::{sync::Mutex, time::sleep};
use tokio_util::sync::CancellationToken;

use self::buckets::Buckets;

mod buckets;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Arc::new(Mutex::new(Terminal::new(backend)?));

    let initial_data = [("B1", 45), ("B2", 72), ("B3", 38)]
        .into_iter()
        .map(|(name, val)| (name.to_string(), val))
        .collect();
    let buckets = Arc::new(Mutex::new(Buckets::new(initial_data)));
    let res = run(terminal.clone(), buckets).await;

    // Restore terminal
    let mut terminal = terminal.lock().await;
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

async fn run(
    terminal: Arc<Mutex<Terminal<CrosstermBackend<Stdout>>>>,
    buckets: Arc<Mutex<Buckets>>,
) -> io::Result<()> {
    let ct = CancellationToken::new();
    let tick_handle = tokio::spawn(run_tick(ct.clone(), buckets.clone()));
    let tui_handle = tokio::spawn(run_tui(ct.clone(), terminal.clone(), buckets.clone()));
    tui_handle.await??;
    tick_handle.await?;
    Ok(())
}

async fn run_tick(ct: CancellationToken, buckets: Arc<Mutex<Buckets>>) {
    loop {
        tokio::select! {
            _ = sleep(Duration::from_secs(1)) => buckets.lock().await.tick().await,
            _ = ct.cancelled() => return,
        }
    }
}

async fn run_tui<B: Backend + Send>(
    ct: CancellationToken,
    terminal: Arc<Mutex<Terminal<B>>>,
    app: Arc<Mutex<Buckets>>,
) -> io::Result<()> {
    let mut reader = crossterm::event::EventStream::new();
    // Start draw_latency at 0 so that we paint the first frame immediately. We then set it to 1 so
    // we draw every second afterwards.
    let mut draw_latency = 0;
    loop {
        tokio::select! {
            _ = ct.cancelled() => return Ok(()),
            _ = sleep(Duration::from_secs(draw_latency)) => {
                let data = app.clone().lock().await.data();
                terminal.lock().await.draw(|f| ui(f, data))?;
                draw_latency = 1;
            },
            maybe_event = reader.next().fuse() => {
                if let Some(event) = maybe_event {
                    handle_event(ct.clone(), event?).await?
                }
            },
        }
    }
}

async fn handle_event(ct: CancellationToken, event: Event) -> io::Result<()> {
    if let Event::Key(key) = event {
        if let KeyCode::Char('q') = key.code {
            ct.cancel();
        }
    }

    Ok(())
}

fn ui(f: &mut Frame, data: Vec<(String, u64)>) {
    // Calculate the width needed for the chart
    // For each bar: width + gap = 9 + 3 = 12 units
    // Last bar doesn't need a gap, plus add some padding and borders
    let bar_width = 9;
    let bar_gap = 3;
    let num_bars = data.len();
    let total_width = (bar_width + bar_gap) * (num_bars - 1) + bar_width + 2; // +2 for borders.

    // Create a centered area with just enough width for our bars
    let area = centered_rect(total_width as u16, 20, f.area());

    // Create the bars for the bar chart:
    let bars = data
        .iter()
        .map(|datum| (datum.0.as_str(), datum.1))
        .collect::<Vec<(&str, u64)>>();

    // Create bar chart
    let bar_chart = BarChart::default()
        .block(
            Block::default()
                .title("Bar Chart Example")
                .borders(Borders::ALL),
        )
        .data(&bars)
        .max(100)
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
