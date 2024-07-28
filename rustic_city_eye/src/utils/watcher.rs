use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::Duration;

pub fn watch_directory(path: PathBuf, tx: Sender<Vec<String>>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut known_files = HashSet::new();
        let mut known_dirs = HashSet::new();
        known_dirs.insert(path.clone());

        fn visit_dirs(
            known_dirs: &mut HashSet<PathBuf>,
            known_files: &mut HashSet<PathBuf>,
            tx: &Sender<Vec<String>>,
        ) {
            let mut new_dirs = HashSet::new();
            for dir in known_dirs.iter() {
                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            let entry_path = entry.path();
                            let metadata = fs::metadata(&entry_path).unwrap();
                            if metadata.is_file() && !known_files.contains(&entry_path) {
                                known_files.insert(entry_path.clone());
                                let tuple = vec![
                                    "New file detected".to_string(),
                                    entry_path.to_str().unwrap().to_string(),
                                ];
                                tx.send(tuple).unwrap();
                            } else if metadata.is_dir() && !known_dirs.contains(&entry_path) {
                                new_dirs.insert(entry_path.clone());
                                let tuple = vec![
                                    "New directory detected".to_string(),
                                    entry_path.to_str().unwrap().to_string(),
                                ];
                                tx.send(tuple).unwrap();
                            }
                        }
                    }
                }
            }
            known_dirs.extend(new_dirs);
        }

        loop {
            visit_dirs(&mut known_dirs, &mut known_files, &tx);
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
                assert!(message[0].contains("New file detected"));
                assert!(message[1].contains("test_file.txt"));
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
                assert!(message[0].contains("New directory detected"));
                assert!(message[1].contains("new_dir"));
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
                assert!(message[0].contains("New file detected"));
                assert!(message[1].contains("test_file.txt"));
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
                assert!(message[0].contains("New directory detected"));
                assert!(message[1].contains("new_dir"));
            }
            Err(_) => {
                panic!("Expected new directory detection message");
            }
        }
    }
    #[test]
    fn test_file_creation_in_subdir() {
        let temp_dir = TempDir::new("test_dir").unwrap();
        let dir_path = temp_dir.path().to_path_buf();

        let (tx, rx) = mpsc::channel();
        let handle = watch_directory(dir_path.clone(), tx);

        // Create a new directory
        let new_dir_path = temp_dir.path().join("new_dir");
        fs::create_dir(&new_dir_path).unwrap();

        // Create a new file in the directory
        let file_path = new_dir_path.join("test_file.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello, world!").unwrap();

        // Wait for the watcher to detect the change, with a timeout
        let mut received_message;
        let mut found_new_file = false;

        for _ in 0..5 {
            received_message = rx.recv_timeout(Duration::from_secs(1));
            match &received_message {
                Ok(message) => {
                    println!("Received message: {:?}", message);
                    if message[0].contains("New file detected")
                        && message[1].contains("new_dir/test_file.txt")
                    {
                        found_new_file = true;
                        break;
                    }
                }
                Err(e) => {
                    println!("Failed to receive message: {:?}", e);
                }
            }
        }

        if !found_new_file {
            panic!("Expected new file detection message");
        }
    }
}
