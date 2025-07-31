use tabled::{
    settings::{object::{Columns, Rows}, Alignment, Modify, Style, Width, formatting::TrimStrategy, themes::Colorization, Color}, Table, Tabled
};
use tabled::settings::object::Segment;
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
    #[tabled(rename = "Total")]
    pub total_tokens: String,
}

impl UsageRow {
    /// 表格的列数
    const COLUMN_COUNT: usize = 7;
    
    /// 获取表格的列数
    pub fn column_count() -> usize {
        Self::COLUMN_COUNT
    }
    
    pub fn from_data(date: String, model: String, usage: Usage) -> Self {
        let input = usage.input_tokens.unwrap_or(0);
        let output = usage.output_tokens.unwrap_or(0);
        let cache_creation = usage.cache_creation_input_tokens.unwrap_or(0);
        let cache_read = usage.cache_read_input_tokens.unwrap_or(0);
        let total = input + output + cache_creation + cache_read;
        
        UsageRow {
            date,
            model,
            input_tokens: Self::format_number(input),
            output_tokens: Self::format_number(output),
            cache_creation_input_tokens: Self::format_number(cache_creation),
            cache_read_input_tokens: Self::format_number(cache_read),
            total_tokens: Self::format_number(total),
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
    
    /// 简化模型名称，去除冗余的前后缀
    fn simplify_model_name(model: &str) -> String {
        // 移除 claude- 前缀
        let without_prefix = model.strip_prefix("claude-").unwrap_or(model);
        
        // 尝试匹配常见模式并简化
        // 模式1: {model}-{version}-{date} 例如: sonnet-4-20250514
        // 模式2: {version}-{model}-{date} 例如: 3-opus-20240229
        
        // 分割成部分
        let parts: Vec<&str> = without_prefix.split('-').collect();
        
        if parts.len() >= 3 {
            // 检查最后一部分是否是日期（8位数字）
            let last_part = parts.last().unwrap();
            if last_part.len() == 8 && last_part.chars().all(|c| c.is_numeric()) {
                // 去掉日期部分
                let without_date = &parts[..parts.len() - 1];
                
                // 重新组合，优化显示
                if without_date.len() == 2 {
                    // 可能是 model-version 或 version-model
                    let first = without_date[0];
                    let second = without_date[1];
                    
                    // 检查哪个是版本号
                    if first.chars().all(|c| c.is_numeric()) {
                        // version-model 格式，如 3-opus
                        format!("{}{}", second, first)
                    } else if second.chars().all(|c| c.is_numeric()) {
                        // model-version 格式，如 sonnet-4
                        format!("{}{}", first, second)
                    } else {
                        // 都不是数字，保持原样
                        without_date.join("-")
                    }
                } else {
                    // 其他情况，直接连接
                    without_date.join("-")
                }
            } else {
                // 最后一部分不是日期，保持原样
                without_prefix.to_string()
            }
        } else {
            // 部分太少，保持原样
            without_prefix.to_string()
        }
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
                rows.push(UsageRow::from_data(date, Self::simplify_model_name(&model), usage));
            } else {
                // 多个模型，需要合并显示
                let mut combined_models = Vec::new();
                let mut combined_input = Vec::new();
                let mut combined_output = Vec::new();
                let mut combined_cache_create = Vec::new();
                let mut combined_cache_read = Vec::new();
                let mut combined_total = Vec::new();
                
                for (model, usage) in models {
                    combined_models.push(Self::simplify_model_name(&model));
                    
                    let input = usage.input_tokens.unwrap_or(0);
                    let output = usage.output_tokens.unwrap_or(0);
                    let cache_creation = usage.cache_creation_input_tokens.unwrap_or(0);
                    let cache_read = usage.cache_read_input_tokens.unwrap_or(0);
                    let total = input + output + cache_creation + cache_read;
                    
                    combined_input.push(UsageRow::format_number(input));
                    combined_output.push(UsageRow::format_number(output));
                    combined_cache_create.push(UsageRow::format_number(cache_creation));
                    combined_cache_read.push(UsageRow::format_number(cache_read));
                    combined_total.push(UsageRow::format_number(total));
                }
                
                rows.push(UsageRow {
                    date,
                    model: combined_models.join("\n"),
                    input_tokens: combined_input.join("\n"),
                    output_tokens: combined_output.join("\n"),
                    cache_creation_input_tokens: combined_cache_create.join("\n"),
                    cache_read_input_tokens: combined_cache_read.join("\n"),
                    total_tokens: combined_total.join("\n"),
                });
            }
        }

        let num_columns = UsageRow::column_count();
        let mut table = Table::new(rows);

        // 应用样式
        table.with(Style::modern());
        
        // 获取终端宽度并调整表格
        if let Some((TermWidth(width), ..)) = terminal_size() {
            let term_width = width as usize;
            
            // 使用终端宽度的80%，最大200
            let table_width = (term_width * 8 / 10).min(200);
            let cell_width = table_width / num_columns;
            // 设置单元格宽度，增大到给定单元格大小
            table.with(Modify::new(Segment::all()).with(Width::increase(cell_width)));
            table.with(Modify::new(Segment::all()).with(Width::wrap(cell_width)));
        } else {
            // 无法获取终端大小时的后备方案
            table.with(Width::wrap(10));
        }

        // 数字列右对齐（从第3列开始，即索引2-6）
        // increase的MinWidth的布局时 需要使用TrimStrategy协助右对齐
        table.with(
            Modify::new(Columns::new(2..num_columns))
                .with(Alignment::right())
                .with(TrimStrategy::Horizontal)

        );
        
        // 为表头行添加背景色
        table.with(Colorization::exact([Color::FG_BRIGHT_GREEN], Rows::new(0..1)));

        println!("\n=== Usage Summary ===");
        println!("{}", table);
    }
}