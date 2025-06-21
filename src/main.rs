use clap::Parser;
use ncurses::*;
use std::thread;
use std::time::Duration;
use sysinfo::{CpuExt, System, SystemExt};

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
        mvprintw(0, 0, "System Monitor - Press 'q' to quit");
        mvprintw(1, 0, &format!("Refresh interval: {}s", args.interval));
        mvprintw(2, 0, "----------------------------------------");

        // Display CPU information
        mvprintw(4, 0, "CPU Usage:");
        let cpus = system.cpus();
        for (i, cpu) in cpus.iter().enumerate() {
            mvprintw(
                5 + i as i32,
                2,
                &format!("CPU {}: {:.1}%", i, cpu.cpu_usage()),
            );
        }

        // Display memory information
        let memory_row = 6 + cpus.len() as i32;
        mvprintw(memory_row, 0, "Memory Usage:");
        mvprintw(
            memory_row + 1,
            2,
            &format!(
                "Total: {:.2} GB",
                system.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0
            ),
        );
        mvprintw(
            memory_row + 2,
            2,
            &format!(
                "Used:  {:.2} GB",
                system.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0
            ),
        );
        mvprintw(
            memory_row + 3,
            2,
            &format!(
                "Free:  {:.2} GB",
                system.free_memory() as f64 / 1024.0 / 1024.0 / 1024.0
            ),
        );

        let memory_percent = (system.used_memory() as f64 / system.total_memory() as f64) * 100.0;
        mvprintw(memory_row + 4, 2, &format!("Usage: {:.1}%", memory_percent));

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
