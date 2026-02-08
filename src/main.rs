mod clock;

use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, RenderDirection, Sparkline, Wrap},
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

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Perf,
    Clock,
}

const CLOCK_COLORS: [Color; 16] = [
    Color::Black,
    Color::Red,
    Color::Green,
    Color::Yellow,
    Color::Blue,
    Color::Magenta,
    Color::Cyan,
    Color::White,
    Color::DarkGray,
    Color::LightRed,
    Color::LightGreen,
    Color::LightYellow,
    Color::LightBlue,
    Color::LightMagenta,
    Color::LightCyan,
    Color::Gray,
];

impl Tab {
    fn name(&self) -> &str {
        match self {
            Tab::Perf => "perf(1)",
            Tab::Clock => "clock(2)",
        }
    }

    fn next(&self) -> Self {
        match self {
            Tab::Perf => Tab::Clock,
            Tab::Clock => Tab::Perf,
        }
    }
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
    cpu_history: Vec<u64>,
    terminal_width: u16,
    current_tab: Tab,
    os_name: String,
    kernel_version: String,
    uptime_days: u64,
    clock_color_index: usize,
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

        let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
        let kernel_version = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());
        let uptime_days = System::uptime() / 3600 / 24;

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
            cpu_history: vec![0; 200],
            terminal_width: 0,
            current_tab: Tab::Perf,
            os_name,
            kernel_version,
            uptime_days,
            clock_color_index: 15,
        }
    }

    fn update(&mut self) {
        self.update_system_stats();
    }

    fn set_terminal_width(&mut self, width: u16) {
        self.terminal_width = width;
        let max_points = self.terminal_width as usize;
        while self.cpu_history.len() > max_points {
            self.cpu_history.pop();
        }
    }

    fn switch_tab(&mut self) {
        self.current_tab = self.current_tab.next();
    }

    fn next_clock_color(&mut self) {
        self.clock_color_index = (self.clock_color_index + 1) % CLOCK_COLORS.len();
    }

    fn prev_clock_color(&mut self) {
        self.clock_color_index = self.clock_color_index.saturating_sub(1) % CLOCK_COLORS.len();
    }

    fn clock_color(&self) -> Color {
        CLOCK_COLORS[self.clock_color_index]
    }

    fn update_system_stats(&mut self) {
        if self.last_update.elapsed() >= self.refresh_interval {
            if self.current_tab == Tab::Perf {
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

                let (total_received, total_transmitted) =
                    self.networks.iter().fold((0, 0), |(rx, tx), (_, data)| {
                        (rx + data.total_received(), tx + data.total_transmitted())
                    });

                let bytes_received = total_received.saturating_sub(self.prev_network_received);
                let bytes_transmitted =
                    total_transmitted.saturating_sub(self.prev_network_transmitted);

                self.network_download_kbps = (bytes_received as f64 / elapsed_secs) / 1024.0;
                self.network_upload_kbps = (bytes_transmitted as f64 / elapsed_secs) / 1024.0;

                self.prev_network_received = total_received;
                self.prev_network_transmitted = total_transmitted;

                let load_avg = System::load_average();
                self.load_avg_1 = load_avg.one;
                self.load_avg_5 = load_avg.five;
                self.load_avg_15 = load_avg.fifteen;

                self.cpu_history.insert(0, self.cpu_usage as u64);
            } else {
                self.system.refresh_cpu_all();
                self.cpu_usage = self.system.global_cpu_usage() as f64;
                self.cpu_history.insert(0, self.cpu_usage as u64);
            }

            let max_points = self.terminal_width as usize;
            while self.cpu_history.len() > max_points {
                self.cpu_history.pop();
            }

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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<(), Box<dyn Error>>
where
    <B as Backend>::Error: 'static,
{
    loop {
        app.update();
        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => return Ok(()),
                KeyCode::Tab => app.switch_tab(),
                KeyCode::Left if app.current_tab == Tab::Clock => app.prev_clock_color(),
                KeyCode::Right if app.current_tab == Tab::Clock => app.next_clock_color(),
                _ => {}
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    app.set_terminal_width(f.area().width);

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    let tab_line = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(12)])
        .split(main_chunks[0]);

    let tab_text = Line::from(vec![Span::styled(
        format!("{} TAB", app.current_tab.name()),
        Style::default().fg(Color::DarkGray),
    )]);

    let tabs = Paragraph::new(tab_text)
        .alignment(Alignment::Right)
        .wrap(Wrap { trim: false });
    f.render_widget(tabs, tab_line[1]);

    match app.current_tab {
        Tab::Perf => render_perf_tab(f, app, main_chunks[1]),
        Tab::Clock => render_clock_tab(f, app, main_chunks[1]),
    }

    let sparkline_data: Vec<u64> = app
        .cpu_history
        .iter()
        .map(|&x| if x < 10 { 10 } else { x })
        .collect();

    let sparkline = Sparkline::default()
        .data(&sparkline_data)
        .max(100)
        .style(Style::default().fg(Color::DarkGray))
        .direction(RenderDirection::RightToLeft);
    f.render_widget(sparkline, main_chunks[2]);
}

fn render_perf_tab(f: &mut Frame, app: &mut App, area: ratatui::prelude::Rect) {
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(area);

    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Length(15),
            Constraint::Min(0),
        ])
        .split(horizontal_chunks[1]);

    let widget_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(vertical_chunks[1]);

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

    let memory_title = format!(" Memory ({:.1}%) ", app.memory_percent);
    let memory_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(memory_title))
        .gauge_style(Style::default().fg(get_gauge_color(app.memory_percent)))
        .percent(app.memory_percent as u16)
        .label(format!("{:.1} GB", app.used_memory_gb));
    f.render_widget(memory_gauge, widget_chunks[1]);

    let swap_title = format!(" Swap ({:.1}%) ", app.swap_percent);
    let swap_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(swap_title))
        .gauge_style(Style::default().fg(get_gauge_color(app.swap_percent)))
        .percent(app.swap_percent as u16)
        .label(format!("{:.1} GB", app.used_swap_gb));

    f.render_widget(swap_gauge, widget_chunks[2]);

    let network_gauge = Paragraph::new(format!(
        "↓ {:.1} ↑ {:.1} KB/s",
        app.network_download_kbps, app.network_upload_kbps
    ))
    .block(Block::default().borders(Borders::ALL).title(" Network "))
    .centered();

    f.render_widget(network_gauge, widget_chunks[3]);

    let info_text = format!(
        "OS: {} | Kernel: {} | Uptime: {} days",
        app.os_name, app.kernel_version, app.uptime_days
    );
    let info_widget = Paragraph::new(info_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));
    f.render_widget(info_widget, widget_chunks[4]);
}

fn render_clock_tab(f: &mut Frame, app: &mut App, area: ratatui::prelude::Rect) {
    clock::render_clock(f, area, app.clock_color());
}
