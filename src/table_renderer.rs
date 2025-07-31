use tabled::{
    settings::{object::{Columns, Segment}, Alignment, Modify, Style, Width}, Table, Tabled
};
use crate::item::Usage;
use terminal_size::{Width as TermWidth, terminal_size};

#[derive(Tabled)]
pub struct UsageRow {
    #[tabled(rename = "Date")]
    pub date: String,
    #[tabled(rename = "Model")]
    pub model: String,
    #[tabled(rename = "Input")]
    pub input_tokens: String,
    #[tabled(rename = "Output")]
    pub output_tokens: String,
    #[tabled(rename = "Cache Create")]
    pub cache_creation_input_tokens: String,
    #[tabled(rename = "Cache Read")]
    pub cache_read_input_tokens: String,
}

impl UsageRow {
    pub fn from_data(date: String, model: String, usage: Usage) -> Self {
        UsageRow {
            date,
            model,
            input_tokens: Self::format_number(usage.input_tokens.unwrap_or(0)),
            output_tokens: Self::format_number(usage.output_tokens.unwrap_or(0)),
            cache_creation_input_tokens: Self::format_number(usage.cache_creation_input_tokens.unwrap_or(0)),
            cache_read_input_tokens: Self::format_number(usage.cache_read_input_tokens.unwrap_or(0)),
        }
    }
    
    fn format_number(n: u32) -> String {
        if n >= 1_000_000 {
            format!("{:.1}M", n as f64 / 1_000_000.0)
        } else if n >= 1_000 {
            format!("{:.1}K", n as f64 / 1_000.0)
        } else {
            n.to_string()
        }
    }
}

pub struct TableRenderer;

impl TableRenderer {
    pub fn new() -> Self {
        Self
    }

    pub fn render_usage_table(&self, data: Vec<((String, String), Usage)>) {
        if data.is_empty() {
            println!("没有可显示的使用数据。");
            return;
        }

        // 按日期分组数据
        use std::collections::BTreeMap;
        let mut grouped_data: BTreeMap<String, Vec<(String, Usage)>> = BTreeMap::new();
        
        for ((model, date), usage) in data {
            // 过滤掉所有值都为0的数据
            if usage.input_tokens.unwrap_or(0) == 0 && 
               usage.output_tokens.unwrap_or(0) == 0 &&
               usage.cache_creation_input_tokens.unwrap_or(0) == 0 &&
               usage.cache_read_input_tokens.unwrap_or(0) == 0 {
                continue;
            }
            grouped_data.entry(date).or_insert_with(Vec::new).push((model, usage));
        }

        // 创建表格行，相同日期的多个模型会合并显示
        let mut rows: Vec<UsageRow> = Vec::new();
        for (date, models) in grouped_data {
            if models.len() == 1 {
                // 只有一个模型，正常显示
                let (model, usage) = models.into_iter().next().unwrap();
                rows.push(UsageRow::from_data(date, model, usage));
            } else {
                // 多个模型，需要合并显示
                let mut combined_models = Vec::new();
                let mut combined_input = Vec::new();
                let mut combined_output = Vec::new();
                let mut combined_cache_create = Vec::new();
                let mut combined_cache_read = Vec::new();
                
                for (model, usage) in models {
                    combined_models.push(model);
                    combined_input.push(UsageRow::format_number(usage.input_tokens.unwrap_or(0)));
                    combined_output.push(UsageRow::format_number(usage.output_tokens.unwrap_or(0)));
                    combined_cache_create.push(UsageRow::format_number(usage.cache_creation_input_tokens.unwrap_or(0)));
                    combined_cache_read.push(UsageRow::format_number(usage.cache_read_input_tokens.unwrap_or(0)));
                }
                
                rows.push(UsageRow {
                    date,
                    model: combined_models.join("\n"),
                    input_tokens: combined_input.join("\n"),
                    output_tokens: combined_output.join("\n"),
                    cache_creation_input_tokens: combined_cache_create.join("\n"),
                    cache_read_input_tokens: combined_cache_read.join("\n"),
                });
            }
        }

        let mut table = Table::new(rows);
        
        // 应用样式
        table.with(Style::modern());
        
        // 数字列右对齐（从第3列开始）
        table.modify(Columns::new(2..6), Alignment::right());
        
        // 获取终端宽度并调整表格
        if let Some((TermWidth(width), ..)) = terminal_size() {
            let term_width = width as usize;
            
            // 使用终端宽度的80%，最大200
            let table_width = (term_width * 8 / 10).min(200);
            let cell_width = table_width / 6;
            // 设置全体表格宽度，当文字超出单元格限制时，自动换行
            table.with(Modify::new(Segment::all()).with(Width::wrap(cell_width)));
            // 设置单元格宽度，增大到给定单元格大小
            table.with(Modify::new(Segment::all()).with(Width::increase(cell_width)));
        } else {
            // 无法获取终端大小时的后备方案
            table.with(Width::wrap(100));
        }

        println!("\n=== Usage Summary ===");
        println!("{}", table);
    }
}