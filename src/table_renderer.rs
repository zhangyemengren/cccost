use tabled::{
    Tabled, Table,
    settings::{Style, Width, Alignment, object::Columns, measurement::Percent, Modify},
};
use tabled::settings::object::Segment;
use tabled::settings::width::Wrap;
use crate::item::Usage;

#[derive(Tabled)]
pub struct UsageRow {
    #[tabled(rename = "Model")]
    pub model: String,
    #[tabled(rename = "Date")]
    pub date: String,
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
    pub fn from_data(model: String, date: String, usage: Usage) -> Self {
        UsageRow {
            model,
            date,
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
            println!("No usage data to display.");
            return;
        }

        // Convert data to table rows
        let rows: Vec<UsageRow> = data.into_iter()
            .map(|((model, date), usage)| UsageRow::from_data(model, date, usage))
            .collect();

        let mut table = Table::new(rows);

        table.with(Style::modern()).with(Width::wrap(100));
        // Right align numeric columns
        table.modify(Columns::new(2..6), Alignment::right());

        println!("\n=== Usage Summary ===");
        println!("{}", table);
    }
}