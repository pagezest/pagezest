use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlogPost {
    pub id: i32,
    pub title: String,
    pub content: String,
    #[serde(skip)]
    pub content_flatbuffer: Vec<u8>,
    #[serde(skip)]
    pub content_cached: Vec<u8>,
    pub slug: String,
    pub created_at: String,
    pub updated_at: String,
}

impl BlogPost {
    pub fn new(slug: &str, title: &str, content: &str, content_flatbuffer: Vec<u8>) -> BlogPost {
        Self {
            id: 0,
            title: title.to_string(),
            content: content.to_string(),
            content_flatbuffer: content_flatbuffer.clone(),
            content_cached: content_flatbuffer,
            slug: slug.to_string(),
            created_at: Utc::now().to_string(),
            updated_at: Utc::now().to_string(),
        }
    }

    pub fn new_with_id(
        id: i32,
        slug: &str,
        title: &str,
        content: &str,
        content_flatbuffer: Vec<u8>,
    ) -> BlogPost {
        Self {
            id,
            title: title.to_string(),
            content: content.to_string(),
            content_flatbuffer: content_flatbuffer.clone(),
            content_cached: content_flatbuffer,
            slug: slug.to_string(),
            created_at: Utc::now().to_string(),
            updated_at: Utc::now().to_string(),
        }
    }
}
