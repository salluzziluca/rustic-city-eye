use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::Duration;

pub fn watch_directory(path: PathBuf, tx: Sender<String>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut known_files = HashSet::new();

        loop {
            match fs::read_dir(&path) {
                Ok(entries) => {
                    let mut current_files = HashSet::new();

                    for entry in entries {
                        if let Ok(entry) = entry {
                            let entry_path = entry.path();
                            current_files.insert(entry_path.clone());
                            if !known_files.contains(&entry_path) {
                                let metadata = fs::metadata(&entry_path).unwrap();
                                if metadata.is_file() {
                                    tx.send(format!("New file detected: {:?}", entry_path))
                                        .unwrap();
                                } else if metadata.is_dir() {
                                    tx.send(format!("New directory detected: {:?}", entry_path))
                                        .unwrap();
                                }
                            }
                        }
                    }

                    known_files = current_files;
                }
                Err(e) => {
                    tx.send(format!("Error reading directory: {}", e)).unwrap();
                }
            }

            thread::sleep(Duration::from_secs(1));
        }
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use std::sync::mpsc;
    use std::time::Duration;
    use tempdir::TempDir;

    #[test]
    fn test_new_file_detection() {
        let temp_dir = TempDir::new("test_dir").unwrap();
        let dir_path = temp_dir.path().to_path_buf();

        let (tx, rx) = mpsc::channel();
        watch_directory(dir_path, tx);

        // Create a new file in the directory
        let file_path = temp_dir.path().join("test_file.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello, world!").unwrap();

        // Wait for the watcher to detect the change, with a timeout
        let received_message = rx.recv_timeout(Duration::from_secs(5));

        // Check if a new file detection message was received
        match received_message {
            Ok(message) => {
                assert!(message.contains("New file detected"));
                assert!(message.contains("test_file.txt"));
            }
            Err(_) => {
                panic!("Expected new file detection message");
            }
        }
    }

    #[test]
    fn test_new_dir_detection() {
        let temp_dir = TempDir::new("test_dir").unwrap();
        let dir_path = temp_dir.path().to_path_buf();

        let (tx, rx) = mpsc::channel();
        watch_directory(dir_path, tx);

        // Create a new directory
        let new_dir_path = temp_dir.path().join("new_dir");
        fs::create_dir(&new_dir_path).unwrap();

        // Wait for the watcher to detect the change, with a timeout
        let received_message = rx.recv_timeout(Duration::from_secs(5));

        // Check if a new directory detection message was received
        match received_message {
            Ok(message) => {
                assert!(message.contains("New directory detected"));
                assert!(message.contains("new_dir"));
            }
            Err(_) => {
                panic!("Expected new directory detection message");
            }
        }
    }
    #[test]
    fn test_file_added_and_not_assumed_as_dir() {
        let temp_dir = TempDir::new("test_dir").unwrap();
        let dir_path = temp_dir.path().to_path_buf();

        let (tx, rx) = mpsc::channel();
        watch_directory(dir_path, tx);

        // Create a new file in the directory
        let file_path = temp_dir.path().join("test_file.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello, world!").unwrap();

        // Wait for the watcher to detect the change, with a timeout
        let received_message = rx.recv_timeout(Duration::from_secs(5));

        // Check if a new file detection message was received
        match received_message {
            Ok(message) => {
                assert!(message.contains("New file detected"));
                assert!(message.contains("test_file.txt"));
            }
            Err(_) => {
                panic!("Expected new file detection message");
            }
        }
    }
    #[test]
    fn test_dir_created_and_not_assumed_as_file() {
        let temp_dir = TempDir::new("test_dir").unwrap();
        let dir_path = temp_dir.path().to_path_buf();

        let (tx, rx) = mpsc::channel();
        watch_directory(dir_path, tx);

        // Create a new directory
        let new_dir_path = temp_dir.path().join("new_dir");
        fs::create_dir(&new_dir_path).unwrap();

        // Wait for the watcher to detect the change, with a timeout
        let received_message = rx.recv_timeout(Duration::from_secs(5));

        // Check if a new directory detection message was received
        match received_message {
            Ok(message) => {
                assert!(message.contains("New directory detected"));
                assert!(message.contains("new_dir"));
            }
            Err(_) => {
                panic!("Expected new directory detection message");
            }
        }
    }
}
