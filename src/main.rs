use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Margin},
    style::{Color, Style},
    symbols,
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame, Terminal,
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};
use sysinfo::System;

#[derive(Parser)]
#[command(name = "zemon")]
#[command(about = "A simple system monitor using ratatui")]
struct Args {
    /// Refresh interval in seconds
    #[arg(short, long, default_value = "2")]
    interval: u64,
}

struct App {
    system: System,
    cpu_usage: f64,
    memory_percent: f64,
    total_memory_gb: f64,
    used_memory_gb: f64,
    refresh_interval: Duration,
    last_update: Instant,
}

impl App {
    fn new(refresh_interval: Duration) -> App {
        let mut system = System::new_all();
        system.refresh_all();

        let cpu_usage = system.global_cpu_usage() as f64;
        let total_memory_gb = system.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
        let used_memory_gb = system.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
        let memory_percent = (system.used_memory() as f64 / system.total_memory() as f64) * 100.0;

        App {
            system,
            cpu_usage,
            memory_percent,
            total_memory_gb,
            used_memory_gb,
            refresh_interval,
            last_update: Instant::now(),
        }
    }

    fn update(&mut self) {
        if self.last_update.elapsed() >= self.refresh_interval {
            self.system.refresh_all();
            self.cpu_usage = self.system.global_cpu_usage() as f64;
            self.total_memory_gb = self.system.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
            self.used_memory_gb = self.system.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
            self.memory_percent =
                (self.system.used_memory() as f64 / self.system.total_memory() as f64) * 100.0;
            self.last_update = Instant::now();
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new(Duration::from_secs(args.interval));

    // Run the app
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
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        app.update();
        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => return Ok(()),
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // CPU
            Constraint::Length(5), // Memory
            Constraint::Min(0),    // Remaining space
        ])
        .split(f.size());

    // CPU Usage
    let cpu_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("🖥️  CPU"))
        .gauge_style(Style::default().fg(Color::Cyan))
        .percent(app.cpu_usage as u16)
        .label(format!("{:.1}%", app.cpu_usage));
    f.render_widget(cpu_gauge, chunks[0]);

    // Memory Usage
    let memory_info = format!(
        "Available: {:.2} GB\nUsed: {:.2} GB",
        app.total_memory_gb, app.used_memory_gb
    );
    let memory_block = Block::default().borders(Borders::ALL).title("🧠 Memory");
    let memory_area = memory_block.inner(chunks[1]);
    f.render_widget(memory_block, chunks[1]);

    let memory_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(2)])
        .split(memory_area);

    let memory_gauge = Gauge::default()
        .gauge_style(Style::default().fg(Color::Green))
        .percent(app.memory_percent as u16)
        .label(format!("{:.1}%", app.memory_percent));
    f.render_widget(memory_gauge, memory_chunks[0]);

    let memory_details = Paragraph::new(memory_info);
    f.render_widget(memory_details, memory_chunks[1]);
}
