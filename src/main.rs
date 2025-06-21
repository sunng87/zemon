use clap::Parser;
use ncurses::*;
use std::thread;
use std::time::Duration;
use sysinfo::System;

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

        // Display CPU information
        mvprintw(4, 0, "CPU Usage:").unwrap();
        let cpu_usage = system.global_cpu_usage();
        mvprintw(5, 2, &format!("Overall: {:.1}%", cpu_usage)).unwrap();

        // Display memory information
        let memory_row = 7;
        mvprintw(memory_row, 0, "Memory Usage:").unwrap();
        let total_memory_gb = system.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
        let used_memory_gb = system.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
        let memory_percent = (system.used_memory() as f64 / system.total_memory() as f64) * 100.0;

        mvprintw(
            memory_row + 1,
            2,
            &format!("Available: {:.2} GB", total_memory_gb),
        )
        .unwrap();
        mvprintw(
            memory_row + 2,
            2,
            &format!("Used: {:.2} GB ({:.1}%)", used_memory_gb, memory_percent),
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
