use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Borders, Gauge, Paragraph},
};

use crate::get_gauge_color;

const REAL_FILESYSTEMS: &[&str] = &[
    "ext2", "ext3", "ext4", "btrfs", "xfs", "zfs", "f2fs", "reiserfs", "reiser4", "jfs", "ufs",
    "nilfs2", "bcachefs", "ntfs", "ntfs3", "vfat", "fat", "fat32", "exfat", "hfs", "hfsplus",
    "apfs",
];

#[derive(Clone)]
pub struct DiskInfo {
    name: String,
    mount_point: String,
    file_system: String,
    is_removable: bool,
    total_gb: f64,
    used_gb: f64,
    percent: f64,
}

pub fn collect_disks(disks: &sysinfo::Disks) -> Vec<DiskInfo> {
    let mut infos: Vec<DiskInfo> = disks
        .list()
        .iter()
        .filter(|d| {
            let fs_type = d.file_system().to_string_lossy().into_owned();
            REAL_FILESYSTEMS.contains(&fs_type.as_str())
        })
        .map(|d| {
            let total = d.total_space();
            let available = d.available_space();
            let used = total.saturating_sub(available);
            let percent = if total > 0 {
                (used as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            DiskInfo {
                name: d.name().to_string_lossy().into_owned(),
                mount_point: d.mount_point().to_string_lossy().into_owned(),
                file_system: d.file_system().to_string_lossy().into_owned(),
                is_removable: d.is_removable(),
                total_gb: total as f64 / 1024.0 / 1024.0 / 1024.0,
                used_gb: used as f64 / 1024.0 / 1024.0 / 1024.0,
                percent,
            }
        })
        .collect();

    infos.sort_by(|a, b| a.mount_point.cmp(&b.mount_point));
    infos
}

pub fn render_disk_tab(f: &mut Frame, area: Rect, disks: &[DiskInfo]) {
    if disks.is_empty() {
        let msg = Paragraph::new("No real disks found")
            .alignment(Alignment::Center)
            .style(Style::default().fg(ratatui::style::Color::DarkGray));
        f.render_widget(msg, area);
        return;
    }

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(area);
    let area = columns[1];

    let constraints = std::iter::once(Constraint::Percentage(20))
        .chain(disks.iter().map(|_| Constraint::Length(3)))
        .chain(std::iter::once(Constraint::Min(0)))
        .collect::<Vec<_>>();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    for (disk, chunk) in disks.iter().zip(chunks.iter().skip(1)) {
        let removable = if disk.is_removable {
            " (removable)"
        } else {
            ""
        };
        let title = format!(
            " {} {} ({}){} ",
            disk.name, disk.mount_point, disk.file_system, removable
        );
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(title))
            .gauge_style(Style::default().fg(get_gauge_color(disk.percent)))
            .percent(disk.percent.min(100.0) as u16)
            .label(format!(
                "{:.1}%  {:.1}/{:.1} GB",
                disk.percent, disk.used_gb, disk.total_gb
            ));
        f.render_widget(gauge, *chunk);
    }
}
