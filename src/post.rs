use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct BlogPost {
    pub title: String,
    pub content: Value,
    pub slug: String,
    pub created_at: String,
}

impl BlogPost {
    pub fn new(slug: &str, title: &str, content: Value) -> BlogPost {
        Self {
            title: title.to_string(),
            content,
            slug: slug.to_string(),
            created_at: Utc::now().to_string(),
        }
    }
}
