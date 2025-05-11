use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct BlogPost {
    pub title: String,
    pub content: Value,
    #[serde(skip)]
    pub content_flatbuffer: Vec<u8>,
    pub slug: String,
    pub created_at: String,
    pub updated_at: String,
}

impl BlogPost {
    pub fn new(slug: &str, title: &str, content: Value, content_flatbuffer: Vec<u8>) -> BlogPost {
        Self {
            title: title.to_string(),
            content,
            content_flatbuffer,
            slug: slug.to_string(),
            created_at: Utc::now().to_string(),
            updated_at: Utc::now().to_string(),
        }
    }
}
