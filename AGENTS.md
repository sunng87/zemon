# Agent Guidelines for zemon

## Build, Lint, and Test Commands

### Build Commands
- `cargo check` - Check if the code compiles without building
- `cargo build` - Build the project in debug mode
- `cargo build --release` - Build optimized release binary

### Linting Commands
- `cargo clippy` - Run Clippy for additional linting and code quality checks
- Always run `cargo clippy` after making changes to ensure code quality

### Test Commands
- `cargo test` - Run all tests
- `cargo test <test_name>` - Run a single test by name
- `cargo test <module_name>::<test_name>` - Run a specific test in a module

### Format Commands
- `cargo fmt` - Format code

Always run `cargo fmt`, `cargo check`, `cargo clippy`, and `cargo test` in
sequence after making changes.

## Code Style Guidelines

### Imports
- Group imports by crate with blank lines between groups
- Order: standard library, external crates, internal modules
- Use `use crate::path::Item` for absolute paths
- Prefer specific imports over glob imports

Example:
```rust
use std::{error::Error, io, time::{Duration, Instant}};

use chrono::Local;
use clap::Parser;
use sysinfo::{Networks, System};
```

### Naming Conventions
- **Structs/Enums**: PascalCase (e.g., `App`, `Args`)
- **Functions/Methods**: snake_case (e.g., `update`, `get_gauge_color`)
- **Variables**: snake_case (e.g., `cpu_usage`, `refresh_interval`)
- **Constants**: SCREAMING_SNAKE_CASE (rarely used, prefer values in config)

### Types and Data Structures
- Use `f64` for floating-point percentages and rates
- Use `u64` for byte counters and history data
- Use `u16` for terminal dimensions
- Use `Duration` for time intervals
- Use `Vec<u64>` for time-series history data

### Error Handling
- Use `Result<(), Box<dyn Error>>` for main function signature
- Use `?` operator for propagating errors
- Use `if let Err(err) = res` pattern for handling terminal errors
- Avoid unwrap() - prefer `?` operator or explicit handling

### Comments
- **NO EXPLANATORY COMMENTS** - The code should be self-explanatory
- Comments are only used for section headers or complex algorithm descriptions
- Use comment sparingly and only when absolutely necessary

### Formatting and Layout
- Use 4-space indentation (Rust default)
- Lines should not exceed 100 characters when possible
- Use builder patterns for complex object construction (e.g., ratatui widgets)
- Chain method calls on separate lines for readability

Example:
```rust
let cpu_gauge = Gauge::default()
    .block(Block::default().borders(Borders::ALL).title(cpu_title))
    .gauge_style(Style::default().fg(get_gauge_color(app.cpu_usage)))
    .percent(app.cpu_usage as u16)
    .label(format!("{:.1}%", app.cpu_usage));
```

### Pattern Matching
- Use match guards for conditional patterns
- Use `saturating_sub()` for safe subtraction of unsigned integers
- Prefer `if let` for single pattern matching

Example:
```rust
match percentage {
    p if p < 25.0 => Color::Blue,
    p if p < 50.0 => Color::Cyan,
    p if p < 75.0 => Color::Yellow,
    _ => Color::Red,
}
```

### String Formatting
- Use `format!` for creating formatted strings
- Use `format!` for display labels with precision (e.g., `"{:.1}%"`)
- Use `Local::now().format("%m-%d %H:%M")` for timestamps

### Struct Initialization
- Initialize struct fields on separate lines for readability
- Initialize computed values before struct creation
- Use field init shorthand when variable names match field names

Example:
```rust
let cpu_usage = system.global_cpu_usage() as f64;
let memory_percent = (system.used_memory() as f64 / system.total_memory() as f64) * 100.0;

App {
    system,
    networks,
    cpu_usage,
    memory_percent,
    refresh_interval,
    last_update: Instant::now(),
}
```

### State Management
- Use `update()` method that checks time elapsed before refreshing
- Store `last_update: Instant` to track update timing
- Separate `update()` (public) from internal update logic
- Use `Duration::from_secs()` for refresh intervals

### UI Rendering
- Use Layout builder pattern for creating widget layouts
- Use constraints like `Constraint::Length()`, `Constraint::Percentage()`, `Constraint::Min()`
- Render widgets in logical order (top to bottom, left to right)
- Use `f.render_widget()` to render widgets to frame

### Terminal Handling
- Always restore terminal state in `main()` function
- Use proper cleanup pattern with `?` operator
- Enable raw mode, alternate screen, and mouse capture at startup
- Disable these features and show cursor before exit

### Time-based Operations
- Use `Instant::now()` for timestamps
- Use `elapsed().as_secs_f64()` for calculating time differences
- Check `elapsed() >= refresh_interval` before updating state

### Project-Specific Notes
- This is a TUI application using ratatui framework
- Optimized for zellij environment (note in Cargo.toml)
- Release profile is configured for minimal binary size
- No unit tests currently exist in the codebase
- Default refresh interval is 2 seconds
