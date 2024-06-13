use std::{
    ffi::OsStr,
    fs::{remove_file, rename},
    path::PathBuf,
};

use chrono::Local;
use krossbar_log_common::ROTATED_LOG_TIMESTAMP_FORMAT;

pub struct Rotator {
    keep_num_files: usize,
    log_location: PathBuf,
}

impl Rotator {
    pub fn new(keep_num_files: usize, log_location: PathBuf) -> Self {
        Self {
            keep_num_files,
            log_location,
        }
    }

    pub fn rotate(&self) -> String {
        let time = Local::now();

        let logs_dir = match self.log_location.parent() {
            Some(log_dir) => log_dir.to_path_buf(),
            _ => {
                eprintln!("Failed to extract log dir from log file path");
                return "".into();
            }
        };

        let rotated_file_name = format!(
            "{}_{}.{}",
            self.log_location
                .as_path()
                .file_stem()
                .unwrap_or(OsStr::new("krossbar_log"))
                .to_string_lossy(),
            time.format(ROTATED_LOG_TIMESTAMP_FORMAT),
            self.log_location
                .as_path()
                .extension()
                .unwrap_or(OsStr::new("log"))
                .to_string_lossy()
        );

        let rotated_file_path = logs_dir.join(rotated_file_name);

        if let Err(err) = rename(self.log_location.clone(), rotated_file_path.clone()) {
            eprintln!("Failed to rotate log file: {}", err.to_string())
        }

        self.remove_old_logs(&logs_dir);

        format!("{}", rotated_file_path.to_string_lossy())
    }

    fn read_log_dir_files(logs_dir: &PathBuf) -> Vec<String> {
        let dir_iter = match logs_dir.read_dir() {
            Ok(dir_iter) => dir_iter,
            Err(err) => {
                eprintln!(
                    "Failed to list log dir: {}. Can't remove old logs",
                    err.to_string()
                );
                return vec![];
            }
        };

        let mut log_files: Vec<String> = dir_iter
            .into_iter()
            .filter_map(|entry| match entry {
                Ok(dir_entry) => {
                    let metadata = dir_entry.metadata();
                    if metadata.is_ok() && metadata.unwrap().is_file() {
                        Some(dir_entry.file_name().to_string_lossy().into_owned())
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .collect();

        log_files.sort();
        log_files
    }

    fn remove_old_logs(&self, logs_dir: &PathBuf) {
        let log_files = Self::read_log_dir_files(logs_dir);

        // Check if have somethig to delete.
        // Note we've just rotated original log file, so we only have rotated files in the dir
        if log_files.len() > self.keep_num_files {
            let num_files_delete = log_files.len() - self.keep_num_files;

            log_files
                .into_iter()
                .take(num_files_delete)
                .for_each(|file| {
                    if let Err(err) = remove_file(logs_dir.join(file).as_path()) {
                        eprintln!("Failed to remove old lof file: {}", err.to_string())
                    }
                })
        }
    }
}
