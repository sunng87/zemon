use clap::Parser;
use ncurses::*;
use std::thread;
use std::time::Duration;
use sysinfo::System;

fn draw_progress_bar(row: i32, col: i32, width: i32, percentage: f64, label: &str) {
    let filled_width = ((percentage / 100.0) * width as f64) as i32;

    mvprintw(row, col, &format!("{}: [", label)).unwrap();

    for i in 0..width {
        if i < filled_width {
            mvaddch(row, col + label.len() as i32 + 3 + i, '█' as u32);
        } else {
            mvaddch(row, col + label.len() as i32 + 3 + i, '░' as u32);
        }
    }

    mvprintw(
        row,
        col + label.len() as i32 + 3 + width,
        &format!("] {:.1}%", percentage),
    )
    .unwrap();
}

#[derive(Parser)]
#[command(name = "sysmon")]
#[command(about = "A simple system monitor using ncurses")]
struct Args {
    /// Refresh interval in seconds
    #[arg(short, long, default_value = "2")]
    interval: u64,
}

fn main() {
    let args = Args::parse();

    // Initialize ncurses
    initscr();
    cbreak();
    noecho();
    nodelay(stdscr(), true);
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    // Initialize system info
    let mut system = System::new_all();

    loop {
        // Refresh system information
        system.refresh_all();

        // Clear screen
        clear();

        // Display header
        mvprintw(0, 0, "System Monitor - Press 'q' to quit").unwrap();
        mvprintw(1, 0, &format!("Refresh interval: {}s", args.interval)).unwrap();
        mvprintw(2, 0, "----------------------------------------").unwrap();

        // Display CPU information with progress bar
        let cpu_usage = system.global_cpu_usage();
        draw_progress_bar(4, 0, 30, cpu_usage as f64, "CPU");

        // Display memory information with progress bar
        let memory_row = 6;
        let total_memory_gb = system.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
        let used_memory_gb = system.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
        let memory_percent = (system.used_memory() as f64 / system.total_memory() as f64) * 100.0;

        draw_progress_bar(memory_row, 0, 30, memory_percent, "Memory");

        mvprintw(
            memory_row + 2,
            2,
            &format!("Available: {:.2} GB", total_memory_gb),
        )
        .unwrap();
        mvprintw(
            memory_row + 3,
            2,
            &format!("Used: {:.2} GB", used_memory_gb),
        )
        .unwrap();

        // Refresh screen
        refresh();

        // Check for quit command
        let ch = getch();
        if ch == 'q' as i32 || ch == 'Q' as i32 {
            break;
        }

        // Sleep for the specified interval
        thread::sleep(Duration::from_secs(args.interval));
    }

    // Clean up ncurses
    endwin();
}
