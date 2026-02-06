use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

#[derive(Copy, Clone, PartialEq)]
pub enum Segment {
    Full,
    Left,
    Center,
    Right,
    Sides,
    Empty,
}

impl Segment {
    pub fn fmt(&self, color: Color) -> Span<'static> {
        let style = Style::default().bg(color);

        match self {
            Self::Full => Span::styled("      ", style),
            Self::Left => Span::styled("  ", style),
            Self::Center => Span::styled("  ", style),
            Self::Right => Span::styled("  ", style),
            Self::Sides => Span::styled("  ", style),
            Self::Empty => Span::raw("      "),
        }
    }
}

pub enum Character {
    Num(u32),
    Colon,
    Empty,
}

impl Character {
    const COLON: [Segment; 5] = [
        Segment::Empty,
        Segment::Center,
        Segment::Empty,
        Segment::Center,
        Segment::Empty,
    ];
    const NUMBERS: [Segment; 50] = [
        Segment::Full,
        Segment::Sides,
        Segment::Sides,
        Segment::Sides,
        Segment::Full, // 0
        Segment::Right,
        Segment::Right,
        Segment::Right,
        Segment::Right,
        Segment::Right, // 1
        Segment::Full,
        Segment::Right,
        Segment::Full,
        Segment::Left,
        Segment::Full, // 2
        Segment::Full,
        Segment::Right,
        Segment::Full,
        Segment::Right,
        Segment::Full, // 3
        Segment::Sides,
        Segment::Sides,
        Segment::Full,
        Segment::Right,
        Segment::Right, // 4
        Segment::Full,
        Segment::Left,
        Segment::Full,
        Segment::Right,
        Segment::Full, // 5
        Segment::Full,
        Segment::Left,
        Segment::Full,
        Segment::Sides,
        Segment::Full, // 6
        Segment::Full,
        Segment::Right,
        Segment::Right,
        Segment::Right,
        Segment::Right, // 7
        Segment::Full,
        Segment::Sides,
        Segment::Full,
        Segment::Sides,
        Segment::Full, // 8
        Segment::Full,
        Segment::Sides,
        Segment::Full,
        Segment::Right,
        Segment::Full, // 9
    ];

    pub fn fmt(&self, color: Color, row: usize) -> Vec<Span<'static>> {
        match self {
            Self::Num(n) => {
                let segment = Self::NUMBERS[*n as usize * 5 + row];
                match segment {
                    Segment::Full => vec![segment.fmt(color), Span::raw(" ")],
                    Segment::Left => vec![segment.fmt(color), Span::raw("     ")],
                    Segment::Center => vec![Span::raw(" "), segment.fmt(color), Span::raw("  ")],
                    Segment::Right => vec![Span::raw("    "), segment.fmt(color), Span::raw(" ")],
                    Segment::Sides => {
                        vec![
                            segment.fmt(color),
                            Span::raw("  "),
                            segment.fmt(color),
                            Span::raw(" "),
                        ]
                    }
                    Segment::Empty => vec![Span::raw("      ")],
                }
            }
            Self::Colon => {
                let segment = Self::COLON[row];
                match segment {
                    Segment::Center => vec![Span::raw("  "), segment.fmt(color), Span::raw("  ")],
                    Segment::Empty => vec![Span::raw("      ")],
                    _ => vec![Span::raw("      ")],
                }
            }
            Self::Empty => vec![Span::raw("      ")],
        }
    }
}

pub fn render_clock(f: &mut Frame, area: ratatui::prelude::Rect, color: Color) {
    let time = chrono::Local::now().format("%H:%M:%S").to_string();
    let date = chrono::Local::now().format("%A, %B %d, %Y").to_string();
    let mut clock_lines = Vec::new();

    for row in 0..5 {
        let mut line_spans = Vec::new();
        for ch in time.chars() {
            let character = if ch == ':' {
                Character::Colon
            } else if ch.is_ascii_digit() {
                Character::Num(ch.to_digit(10).unwrap())
            } else {
                Character::Empty
            };
            line_spans.extend(character.fmt(color, row));
        }
        clock_lines.push(Line::from(line_spans));
    }

    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Length(5),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(area);

    let clock_widget = Paragraph::new(clock_lines).alignment(ratatui::layout::Alignment::Center);
    f.render_widget(clock_widget, vertical_chunks[1]);

    let date_widget = Paragraph::new(date)
        .alignment(ratatui::layout::Alignment::Center)
        .style(Style::default().fg(color));
    f.render_widget(date_widget, vertical_chunks[3]);
}
