use rayon::prelude::*;
use std::fs;
use std::path::PathBuf;
use serde_json::Value;
use crate::item::{Item, Usage, LogEntry};
use dashmap::DashMap;

pub struct FileProcessor {
    directory: PathBuf,
    // 使用 DashMap 替代 Mutex<HashMap>，提供更细粒度的锁
    collected_items: DashMap<(String, String), Usage>, // (模型, 时间戳键) -> 使用量
}

impl FileProcessor {
    pub fn new(directory: PathBuf) -> Self {
        Self { 
            directory,
            collected_items: DashMap::new(),
        }
    }

    pub fn process_files(&self) -> Vec<((String, String), Usage)> {
        if !self.directory.exists() {
            println!("目录 {} 不存在", self.directory.display());
            return Vec::new();
        }

        // 获取所有子目录
        let subdirs: Vec<_> = match fs::read_dir(&self.directory) {
            Ok(entries) => entries
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .filter(|path| path.is_dir())
                .collect(),
            Err(e) => {
                eprintln!("读取目录失败: {}", e);
                return Vec::new();
            }
        };

        // 从所有子目录收集所有文件
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

        // 并行处理文件
        all_files.par_iter().for_each(|file_path| {
            self.process_file(file_path);
        });
        
        // 返回合并后的结果
        self.get_merged_results()
    }

    fn process_file(&self, file_path: &PathBuf) {
        match fs::read_to_string(file_path) {
            Ok(content) => {
                // 检查文件是否为JSON
                if file_path.extension().and_then(|s| s.to_str()) == Some("json") ||
                   file_path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                    self.print_json_content(&content);
                }
            }
            Err(e) => {
                eprintln!("读取文件 {} 出错: {}", file_path.display(), e);
            }
        }
    }

    fn print_json_content(&self, content: &str) {
        // 通过尝试解析第一行来检查是否为JSONL文件
        let lines: Vec<&str> = content.lines().collect();
        
        if lines.is_empty() {
            return;
        }
        
        // 尝试将第一个非空行解析为JSON
        let first_line = lines.iter().find(|line| !line.trim().is_empty());
        
        if let Some(line) = first_line {
            if serde_json::from_str::<Value>(line).is_ok() {
                // 这是JSONL格式 - 逐行处理
                for line in content.lines() {
                    if line.trim().is_empty() {
                        continue;
                    }
                    
                    match serde_json::from_str::<Value>(line) {
                        Ok(json) => {
                            self.print_json_value(&json, 1);
                        }
                        Err(_) => {
                            // 静默跳过无效行
                        }
                    }
                }
            } else {
                // 尝试作为常规JSON解析
                match serde_json::from_str::<Value>(content) {
                    Ok(json) => {
                        self.print_json_value(&json, 0);
                    }
                    Err(_) => {
                        // 静默跳过无效的JSON
                    }
                }
            }
        }
    }

    fn print_json_value(&self, value: &Value, _indent: usize) {
        // 尝试反序列化为LogEntry
        if let Ok(log_entry) = serde_json::from_value::<LogEntry>(value.clone()) {
            if let Some(item) = Item::from_log_entry(log_entry) {
                self.collect_item(item);
            }
        }
    }
    
    fn collect_item(&self, item: Item) {
        let key = (item.model.clone(), item.get_timestamp_key());
        
        if let Some(usage) = item.usage {
            // DashMap 提供了更高效的并发访问
            self.collected_items
                .entry(key)
                .and_modify(|existing| *existing = existing.clone() + usage.clone())
                .or_insert(usage);
        }
    }
    
    fn get_merged_results(&self) -> Vec<((String, String), Usage)> {
        // 直接从 DashMap 转换为 Vec，无需锁
        let mut sorted_items: Vec<_> = self.collected_items
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();
        
        // 按模型和时间戳排序
        sorted_items.sort_by(|a, b| a.0.cmp(&b.0));
        
        sorted_items
    }
}