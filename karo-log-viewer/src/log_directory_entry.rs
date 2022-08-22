use std::{cmp::Ordering, path::PathBuf};

use chrono::NaiveDateTime;

#[derive(Eq, PartialEq)]
pub enum LogFileType {
    Rotated(NaiveDateTime),
    Live,
}

#[derive(Eq, PartialEq)]
pub struct LogFileEntry {
    pub log_file_name: String,
    pub full_path: PathBuf,
    pub log_type: LogFileType,
}

impl Ord for LogFileEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self.log_type, &other.log_type) {
            (LogFileType::Live, LogFileType::Live) => Ordering::Equal,
            (LogFileType::Rotated(_), LogFileType::Live) => Ordering::Less,
            (LogFileType::Live, LogFileType::Rotated(_)) => Ordering::Greater,
            (LogFileType::Rotated(this), LogFileType::Rotated(other)) => this.cmp(other),
        }
    }
}

impl PartialOrd for LogFileEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
