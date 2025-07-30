mod file_processor;
mod table_renderer;
mod item;

use std::path::PathBuf;
use file_processor::FileProcessor;
use table_renderer::TableRenderer;

fn main() {
    // Process files from ~/.claude/projects
    let home_dir = std::env::var("HOME").unwrap_or_else(|_| String::from("~"));
    let claude_projects_dir = PathBuf::from(home_dir).join(".claude/projects");
    
    let file_processor = FileProcessor::new(claude_projects_dir);
    let usage_data = file_processor.process_files();
    
    // Render the usage table
    let table_renderer = TableRenderer::new();
    table_renderer.render_usage_table(usage_data);
}