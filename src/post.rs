use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::uuid::generate_uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlogPost {
    pub id: String,
    pub title: String,
    #[serde(skip)]
    pub content_flatbuffer: Vec<u8>,
    pub content_md: String,
    pub slug: String,
    pub created_at: String,
    pub updated_at: String,
}

impl BlogPost {
    pub fn new(slug: &str, title: &str, content_md: &str, content_flatbuffer: Vec<u8>) -> BlogPost {
        Self {
            id: generate_uuid().to_string(),
            title: title.to_string(),
            content_md: content_md.to_string(),
            content_flatbuffer,
            slug: slug.to_string(),
            created_at: Utc::now().to_string(),
            updated_at: Utc::now().to_string(),
        }
    }

    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "slug": self.slug,
            "title": self.title,
            "content_md": self.content_md,
            "created_at": self.created_at,
            "updated_at": self.updated_at,
        })
    }
}
