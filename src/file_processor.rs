use rayon::prelude::*;
use std::fs;
use std::path::PathBuf;
use serde_json::Value;
use crate::item::{Item, Usage, LogEntry};
use std::collections::HashMap;
use std::sync::Mutex;

pub struct FileProcessor {
    directory: PathBuf,
    collected_items: Mutex<HashMap<(String, String), Usage>>, // (model, timestamp_key) -> Usage
}

impl FileProcessor {
    pub fn new(directory: PathBuf) -> Self {
        Self { 
            directory,
            collected_items: Mutex::new(HashMap::new()),
        }
    }

    pub fn process_files(&self) -> Vec<((String, String), Usage)> {
        if !self.directory.exists() {
            println!("Directory {} does not exist", self.directory.display());
            return Vec::new();
        }

        // Get all subdirectories
        let subdirs: Vec<_> = match fs::read_dir(&self.directory) {
            Ok(entries) => entries
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .filter(|path| path.is_dir())
                .collect(),
            Err(e) => {
                eprintln!("Failed to read directory: {}", e);
                return Vec::new();
            }
        };

        // Collect all files from all subdirectories
        let all_files: Vec<_> = subdirs
            .par_iter()
            .flat_map(|dir| {
                fs::read_dir(dir)
                    .ok()
                    .into_iter()
                    .flat_map(|entries| entries)
                    .filter_map(|entry| entry.ok())
                    .map(|entry| entry.path())
                    .filter(|path| path.is_file())
                    .collect::<Vec<_>>()
            })
            .collect();

        // Process files in parallel
        all_files.par_iter().for_each(|file_path| {
            self.process_file(file_path);
        });
        
        // Return merged results
        self.get_merged_results()
    }

    fn process_file(&self, file_path: &PathBuf) {
        match fs::read_to_string(file_path) {
            Ok(content) => {
                // Check if file is JSON
                if file_path.extension().and_then(|s| s.to_str()) == Some("json") ||
                   file_path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                    self.print_json_content(&content);
                }
            }
            Err(e) => {
                eprintln!("Error reading file {}: {}", file_path.display(), e);
            }
        }
    }

    fn print_json_content(&self, content: &str) {
        // Check if it's a JSONL file by trying to parse the first line
        let lines: Vec<&str> = content.lines().collect();
        
        if lines.is_empty() {
            return;
        }
        
        // Try to parse first non-empty line as JSON
        let first_line = lines.iter().find(|line| !line.trim().is_empty());
        
        if let Some(line) = first_line {
            if serde_json::from_str::<Value>(line).is_ok() {
                // It's JSONL format - process line by line
                for line in content.lines() {
                    if line.trim().is_empty() {
                        continue;
                    }
                    
                    match serde_json::from_str::<Value>(line) {
                        Ok(json) => {
                            self.print_json_value(&json, 1);
                        }
                        Err(_) => {
                            // Silently skip invalid lines
                        }
                    }
                }
            } else {
                // Try to parse as regular JSON
                match serde_json::from_str::<Value>(content) {
                    Ok(json) => {
                        self.print_json_value(&json, 0);
                    }
                    Err(_) => {
                        // Silently skip invalid JSON
                    }
                }
            }
        }
    }

    fn print_json_value(&self, value: &Value, _indent: usize) {
        // Try to deserialize as LogEntry
        if let Ok(log_entry) = serde_json::from_value::<LogEntry>(value.clone()) {
            if let Some(item) = Item::from_log_entry(log_entry) {
                self.collect_item(item);
            }
        }
    }
    
    fn collect_item(&self, item: Item) {
        let key = (item.model.clone(), item.get_timestamp_key());
        
        if let Some(usage) = item.usage {
            let mut items = self.collected_items.lock().unwrap();
            
            items.entry(key)
                .and_modify(|existing| *existing = existing.clone() + usage.clone())
                .or_insert(usage);
        }
    }
    
    fn get_merged_results(&self) -> Vec<((String, String), Usage)> {
        let items = self.collected_items.lock().unwrap();
        
        // Sort by model and timestamp
        let mut sorted_items: Vec<_> = items.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        sorted_items.sort_by(|a, b| a.0.cmp(&b.0));
        
        sorted_items
    }
}