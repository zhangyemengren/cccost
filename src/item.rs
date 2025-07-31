use std::fmt;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::ops::Add;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub message: Message,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message {
    pub model: Option<String>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Item {
    pub model: String,
    pub timestamp: String,
    #[serde(default)]
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Usage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u32>,
}

impl Item {
    pub fn from_log_entry(entry: LogEntry) -> Option<Self> {
        entry.message.model.map(|model| Item {
            model,
            timestamp: entry.timestamp,
            usage: entry.message.usage,
        })
    }
    
    pub fn get_timestamp_key(&self) -> String {
        // 解析时间戳并格式化为同一天（移除时间）
        if let Ok(dt) = self.timestamp.parse::<DateTime<Utc>>() {
            dt.format("%Y-%m-%d").to_string()
        } else {
            self.timestamp.clone()
        }
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Model: {}, Date: {}", self.model, self.get_timestamp_key())?;
        
        if let Some(ref usage) = self.usage {
            write!(f, ", Usage: {}", usage)?;
        }
        
        Ok(())
    }
}

impl fmt::Display for Usage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        
        if let Some(input) = self.input_tokens {
            parts.push(format!("input: {}", input));
        }
        
        if let Some(output) = self.output_tokens {
            parts.push(format!("output: {}", output));
        }
        
        if let Some(cache_creation) = self.cache_creation_input_tokens {
            parts.push(format!("cache_creation: {}", cache_creation));
        }
        
        if let Some(cache_read) = self.cache_read_input_tokens {
            parts.push(format!("cache_read: {}", cache_read));
        }
        
        write!(f, "{{{}}}", parts.join(", "))
    }
}

impl Add for Usage {
    type Output = Usage;

    fn add(self, other: Usage) -> Usage {
        Usage {
            input_tokens: match (self.input_tokens, other.input_tokens) {
                (Some(a), Some(b)) => Some(a + b),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            },
            output_tokens: match (self.output_tokens, other.output_tokens) {
                (Some(a), Some(b)) => Some(a + b),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            },
            cache_creation_input_tokens: match (self.cache_creation_input_tokens, other.cache_creation_input_tokens) {
                (Some(a), Some(b)) => Some(a + b),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            },
            cache_read_input_tokens: match (self.cache_read_input_tokens, other.cache_read_input_tokens) {
                (Some(a), Some(b)) => Some(a + b),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            },
        }
    }
}