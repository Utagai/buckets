use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::{FutureExt, StreamExt};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{BarChart, Block, Borders, List, ListItem},
    Frame, Terminal,
};
use std::{
    error::Error,
    io::{self, Stdout},
    sync::Arc,
    time::Duration,
};
use tokio::{
    sync::{mpsc, Mutex},
    time::sleep,
};
use tokio_util::sync::CancellationToken;

use self::{
    actuator::{Actuator, FinalControlElement},
    buckets::{n_buckets::NBuckets, BucketType, Buckets},
    cli::Args,
    controller::Controller,
    events::Events,
    sensor::Sensor,
};

mod actuator;
mod buckets;
mod cli;
mod controller;
mod events;
mod policy;
mod sensor;

// Updated main function
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments
    let args = Args::parse();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Arc::new(Mutex::new(Terminal::new(backend)?));

    // Use the parsed initial data
    let initial_data = args.initial_data;

    let events = Arc::new(Mutex::new(Events::new()));

    // Create the appropriate bucket type based on args
    let buckets = match args.bucket_type {
        BucketType::NBuckets => Arc::new(Mutex::new(NBuckets::new(initial_data))),
    };

    const CONTROL_SIGNAL_BUFFER_SIZE: usize = 10;
    let (control_signal_tx, control_signal_rx) = mpsc::channel(CONTROL_SIGNAL_BUFFER_SIZE);

    // Use the selected policy
    let controller = Arc::new(Controller::new(
        args.policy,
        buckets.clone(),
        events.clone(),
        control_signal_tx,
    ));

    let actuator = Arc::new(Mutex::new(Actuator::new(
        buckets.clone(),
        events.clone(),
        control_signal_rx,
    )));

    let res = run(
        terminal.clone(),
        events.clone(),
        buckets,
        controller,
        actuator,
    )
    .await;

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

async fn run<S: Buckets + Sensor + FinalControlElement + Send + 'static>(
    terminal: Arc<Mutex<Terminal<CrosstermBackend<Stdout>>>>,
    events: Arc<Mutex<Events>>,
    buckets: Arc<Mutex<NBuckets>>,
    controller: Arc<Controller<S>>,
    actuator: Arc<Mutex<Actuator<S>>>,
) -> Result<()> {
    let ct = CancellationToken::new();
    let fill_handle = tokio::spawn(run_fill(ct.clone(), events.clone(), buckets.clone()));
    let tui_handle = tokio::spawn(run_tui(
        ct.clone(),
        events,
        terminal.clone(),
        buckets.clone(),
    ));
    let controller_handle = tokio::spawn(run_control_loop(ct.clone(), controller.clone()));
    let actuator_handle = tokio::spawn(run_actuator_loop(ct.clone(), actuator.clone()));
    tui_handle.await??;
    fill_handle.await?;
    controller_handle.await??;
    actuator_handle.await??;
    Ok(())
}

async fn run_fill<B: Buckets>(
    ct: CancellationToken,
    events: Arc<Mutex<Events>>,
    buckets: Arc<Mutex<B>>,
) {
    loop {
        tokio::select! {
            _ = sleep(Duration::from_secs(1)) => {
                let (bucket, change, new_val) = buckets.lock().await.fill();
                events.lock().await.add(events::EventSource::Filler, format!("filled +{} to bucket {} => {}", change, bucket, new_val));
            },
            _ = ct.cancelled() => return,
        }
    }
}

async fn run_tui<B: Backend + Send>(
    ct: CancellationToken,
    events: Arc<Mutex<Events>>,
    terminal: Arc<Mutex<Terminal<B>>>,
    app: Arc<Mutex<NBuckets>>,
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
                let lines = events
                    .lock()
                    .await
                    .get_all()
                    .iter()
                    .map(|event| {
                        Line::from(vec![Span::styled(
                            format!("{} | ", event.timestamp.format("%H:%M:%S")),
                            Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD),
                        ),
                            Span::styled(
                            format!("{} ", event.source),
                            Style::default().fg(event.source.color()).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!("{}", event.message),
                            Style::default().fg(Color::White).add_modifier(Modifier::ITALIC),
                        )])
                    })
                    .collect();
                terminal.lock().await.draw(|f| ui(f, data, &lines))?;
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

async fn run_control_loop<S: Sensor + Send + 'static>(
    ct: CancellationToken,
    controller: Arc<Controller<S>>,
) -> Result<()> {
    const CONTROLLER_RUN_LATENCY: u64 = 1;
    loop {
        tokio::select! {
            _ = sleep(Duration::from_secs(CONTROLLER_RUN_LATENCY)) => controller.run().await?,
            _ = ct.cancelled() => return Ok(()),
        }
    }
}

async fn run_actuator_loop<B: FinalControlElement + Send + 'static>(
    ct: CancellationToken,
    actuator: Arc<Mutex<Actuator<B>>>,
) -> Result<()> {
    const ACTUATOR_RUN_LATENCY: u64 = 1;
    loop {
        tokio::select! {
            _ = sleep(Duration::from_secs(ACTUATOR_RUN_LATENCY)) => actuator.lock().await.run().await?,
            _ = ct.cancelled() => return Ok(()),
        }
    }
}

fn ui(f: &mut Frame, data: Vec<(String, u64)>, events: &Vec<Line<'_>>) {
    // Calculate the width needed for the chart
    // For each bar: width + gap = 9 + 3 = 12 units
    // Last bar doesn't need a gap, plus add some padding and borders
    let bar_width = 9;
    let bar_gap = 3;
    let num_bars = data.len();
    let chart_width = (bar_width + bar_gap) * (num_bars - 1) + bar_width + 2; // +2 for borders

    // Calculate the full width for the layout (wider for event log)
    let total_layout_width = (chart_width + 20).max((f.area().width - 10) as usize); // At least 20 units wider than chart, but respect screen size

    // Create the main area with the calculated width
    let main_area = centered_rect(total_layout_width as u16, 30, f.area());

    // Split the main area into two chunks vertically - top for chart, bottom for events
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(20), // Chart height stays the same
            Constraint::Min(10),    // Event log takes remaining space (min 10)
        ])
        .split(main_area);

    // For the top chunk, create a centered area for the bar chart with its specific width
    let chart_area = centered_rect_horizontal(
        chart_width as u16,
        vertical_chunks[0].height,
        vertical_chunks[0],
    );

    // Event log uses the full width of the bottom chunk
    let event_log_area = vertical_chunks[1];

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

    // Create the event log widget with styled text
    let events_list = List::new(
        events
            .iter()
            .map(|spans| ListItem::new(spans.clone()))
            .collect::<Vec<ListItem>>(),
    )
    .block(Block::default().title("Event Log").borders(Borders::ALL))
    .highlight_style(Style::default().add_modifier(Modifier::BOLD))
    .highlight_symbol(">> ");

    // Render both widgets
    f.render_widget(bar_chart, chart_area);
    f.render_widget(events_list, event_log_area);
}

// Add this helper function for horizontal centering with specific width
fn centered_rect_horizontal(width: u16, height: u16, r: Rect) -> Rect {
    let horizontal_padding = (r.width.saturating_sub(width)) / 2;
    Rect::new(
        r.x + horizontal_padding,
        r.y,
        width.min(r.width),
        height.min(r.height),
    )
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
