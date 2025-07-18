use chrono::Local;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame, Terminal,
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};
use sysinfo::{Networks, System};

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
    networks: Networks,
    cpu_usage: f64,
    memory_percent: f64,
    swap_percent: f64,
    used_memory_gb: f64,
    used_swap_gb: f64,
    network_upload_kbps: f64,
    network_download_kbps: f64,
    prev_network_received: u64,
    prev_network_transmitted: u64,
    refresh_interval: Duration,
    last_update: Instant,
    load_avg_1: f64,
    load_avg_5: f64,
    load_avg_15: f64,
}

fn get_gauge_color(percentage: f64) -> Color {
    match percentage {
        p if p < 25.0 => Color::Blue,
        p if p < 50.0 => Color::Cyan,
        p if p < 75.0 => Color::Yellow,
        _ => Color::Red,
    }
}

impl App {
    fn new(refresh_interval: Duration) -> App {
        let mut system = System::new_all();
        system.refresh_all();
        let networks = Networks::new_with_refreshed_list();

        let cpu_usage = system.global_cpu_usage() as f64;
        let used_memory_gb = system.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
        let memory_percent = (system.used_memory() as f64 / system.total_memory() as f64) * 100.0;
        let swap_percent = (system.used_swap() as f64 / system.total_swap() as f64) * 100.0;
        let used_swap_gb = system.used_swap() as f64 / 1024.0 / 1024.0 / 1024.0;

        // Get initial network stats
        let (total_received, total_transmitted) =
            networks.iter().fold((0, 0), |(rx, tx), (_, data)| {
                (rx + data.total_received(), tx + data.total_transmitted())
            });

        let load_avg = System::load_average();

        App {
            system,
            networks,
            cpu_usage,
            memory_percent,
            swap_percent,
            used_memory_gb,
            used_swap_gb,
            network_upload_kbps: 0.0,
            network_download_kbps: 0.0,
            prev_network_received: total_received,
            prev_network_transmitted: total_transmitted,
            refresh_interval,
            last_update: Instant::now(),
            load_avg_1: load_avg.one,
            load_avg_5: load_avg.five,
            load_avg_15: load_avg.fifteen,
        }
    }

    fn update(&mut self) {
        if self.last_update.elapsed() >= self.refresh_interval {
            self.system.refresh_all();
            self.networks.refresh(true);

            let elapsed_secs = self.last_update.elapsed().as_secs_f64();

            self.cpu_usage = self.system.global_cpu_usage() as f64;
            self.used_memory_gb = self.system.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
            self.memory_percent =
                (self.system.used_memory() as f64 / self.system.total_memory() as f64) * 100.0;
            self.swap_percent =
                (self.system.used_swap() as f64 / self.system.total_swap() as f64) * 100.0;
            self.used_swap_gb = self.system.used_swap() as f64 / 1024.0 / 1024.0 / 1024.0;

            // Calculate network speeds
            let (total_received, total_transmitted) =
                self.networks.iter().fold((0, 0), |(rx, tx), (_, data)| {
                    (rx + data.total_received(), tx + data.total_transmitted())
                });

            let bytes_received = total_received.saturating_sub(self.prev_network_received);
            let bytes_transmitted = total_transmitted.saturating_sub(self.prev_network_transmitted);

            self.network_download_kbps = (bytes_received as f64 / elapsed_secs) / 1024.0;
            self.network_upload_kbps = (bytes_transmitted as f64 / elapsed_secs) / 1024.0;

            self.prev_network_received = total_received;
            self.prev_network_transmitted = total_transmitted;

            // Update load averages
            let load_avg = System::load_average();
            self.load_avg_1 = load_avg.one;
            self.load_avg_5 = load_avg.five;
            self.load_avg_15 = load_avg.fifteen;

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
    // Create horizontal centering with padding
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20), // Left padding
            Constraint::Percentage(60), // Center content
            Constraint::Percentage(20), // Right padding
        ])
        .split(f.area());

    // Create vertical centering with padding
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20), // Top padding
            Constraint::Length(15),     // Content height (4 widgets + borders)
            Constraint::Percentage(20), // Bottom padding
        ])
        .split(horizontal_chunks[1]);

    // Create the widget layout within the centered area
    let widget_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // CPU
            Constraint::Length(3), // Memory
            Constraint::Length(3), // Swap
            Constraint::Length(3), // Network
            Constraint::Length(1), // Time
        ])
        .split(vertical_chunks[1]);

    // CPU Usage
    let cpu_title = format!(
        " CPU ({:.2} {:.2} {:.2}) ",
        app.load_avg_1, app.load_avg_5, app.load_avg_15
    );
    let cpu_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(cpu_title))
        .gauge_style(Style::default().fg(get_gauge_color(app.cpu_usage)))
        .percent(app.cpu_usage as u16)
        .label(format!("{:.1}%", app.cpu_usage));
    f.render_widget(cpu_gauge, widget_chunks[0]);

    // Memory Usage
    let memory_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Memory "))
        .gauge_style(Style::default().fg(get_gauge_color(app.memory_percent)))
        .percent(app.memory_percent as u16)
        .label(format!("{:.1} GB", app.used_memory_gb));
    f.render_widget(memory_gauge, widget_chunks[1]);

    // Swap Usage
    let swap_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Swap "))
        .gauge_style(Style::default().fg(get_gauge_color(app.swap_percent)))
        .percent(app.swap_percent as u16)
        .label(format!("{:.1} GB", app.used_swap_gb));

    f.render_widget(swap_gauge, widget_chunks[2]);

    // Network Usage
    let network_gauge = Paragraph::new(format!(
        "↓ {:.1} ↑ {:.1} KB/s",
        app.network_download_kbps, app.network_upload_kbps
    ))
    .block(Block::default().borders(Borders::ALL).title(" Network "))
    .centered();

    f.render_widget(network_gauge, widget_chunks[3]);

    // Current Time
    let current_time = Local::now().format("%m-%d %H:%M").to_string();
    let time_widget = Paragraph::new(current_time)
        .centered()
        .style(Style::default().bold());
    f.render_widget(time_widget, widget_chunks[4]);
}
