use std::path::PathBuf;

use chrono::NaiveDateTime;
use karo_log_common::ROTATED_LOG_TIMESTAMP_FORMAT;
use log::{debug, error, warn};
use regex::Regex;

use crate::log_directory_entry::{LogFile, LogFileType};

pub struct DirectoryReader;

impl DirectoryReader {
    /// Read log directory to find all log files
    pub fn read_dir_logs(log_location: &str) -> Vec<LogFile> {
        let mut result = vec![];

        let log_path = PathBuf::from(log_location);

        if !log_path.exists() {
            error!("Log file at '{}' doesn't exist", log_path.display());
        }

        let live_log_name: String = log_path.file_name().unwrap().to_string_lossy().into();
        debug!("Live log file name: {}", live_log_name);

        let rotated_log_regex = match Self::get_rotated_log_regex(&live_log_name) {
            Some(regx) => regx,
            None => {
                error!("Failed to make a regex for rotated logs");
                return result;
            }
        };

        let log_dir = log_path.parent().unwrap().to_owned();
        let dir_entries = match log_dir.read_dir() {
            Ok(entries) => entries,
            Err(err) => {
                error!("Failed to read log directory: {}", err.to_string());
                return result;
            }
        };

        for entry in dir_entries {
            match entry {
                Ok(dir_entry) => {
                    let log_file_name: String = dir_entry.file_name().to_string_lossy().into();

                    // Skip dirs and live log
                    if !dir_entry.file_type().map(|e| e.is_file()).unwrap_or(false)
                        || live_log_name == log_file_name
                    {
                        continue;
                    }

                    debug!("Potential rotated log file: {}", log_file_name);

                    // Try to match filename with rotated log regex
                    if let Some(caps) = rotated_log_regex.captures(&log_file_name) {
                        let rotated_log_datetime = &caps[1];

                        debug!("Rotated log timestamp: {}", rotated_log_datetime);

                        // Parse rotation timestamp
                        match NaiveDateTime::parse_from_str(
                            rotated_log_datetime,
                            ROTATED_LOG_TIMESTAMP_FORMAT,
                        ) {
                            Ok(ts) => {
                                debug!("Succesfully parsed rotated log '{}'", log_file_name);

                                result.push(LogFile {
                                    log_file_name,
                                    log_type: LogFileType::Rotated(ts),
                                })
                            }
                            Err(_) => {
                                error!(
                                    "Failed to parse rotated log timestamp '{}'",
                                    rotated_log_datetime
                                )
                            }
                        }
                    }
                }
                Err(err) => {
                    warn!("Failed to read log dir entry: {}", err.to_string())
                }
            }
        }

        if result.is_empty() {
            warn!("No rotated logs in the log directory");
        }

        result.sort();
        result.push(LogFile {
            log_file_name: live_log_name,
            log_type: LogFileType::Live,
        });

        result
    }

    /// Make rotated log regex out of the live log file name.
    /// The regex allows to parse rotated log timestamp
    fn get_rotated_log_regex(live_log_name: &str) -> Option<Regex> {
        if let Some((file_name, ext)) = live_log_name.rsplit_once('.') {
            let rotate_file_regex = format!(r"{}_([^\.]+)\.{}", file_name, ext);

            Regex::new(&rotate_file_regex).ok()
        } else {
            warn!(
                "Failed to split live log name '{}' into file_name and extension",
                live_log_name
            );
            None
        }
    }
}
