use std::collections::VecDeque;
use std::path::PathBuf;

use crate::live_log_file::LiveLogFile;
use crate::log_directory_entry::LogFileType;
use crate::log_directory_reader::DirectoryReader;
use crate::log_file_trait::LogFile;
use crate::rotated_log_file::RotatedLogFile;

/// Log files registry witch implements cross-file browsing
pub struct LogRegistry {
    pub rotated_log_files: VecDeque<Box<dyn LogFile>>,
    /// Currently visible window inside [log_files]
    pub current_window: (usize, usize),
}

impl LogRegistry {
    pub fn new(log_location: &str) -> Self {
        let rotated_log_files: VecDeque<Box<dyn LogFile>> =
            DirectoryReader::read_dir_logs(log_location)
                .into_iter()
                .map(|log_entry| {
                    let file_path = PathBuf::from(log_entry.log_file_name);

                    match log_entry.log_type {
                        LogFileType::Rotated(ts) => {
                            Box::new(RotatedLogFile::new(file_path, ts)) as Box<dyn LogFile>
                        }
                        LogFileType::Live => {
                            Box::new(LiveLogFile::new(file_path)) as Box<dyn LogFile>
                        }
                    }
                })
                .collect();

        Self {
            rotated_log_files,
            current_window: (0, 0),
        }
    }
}
