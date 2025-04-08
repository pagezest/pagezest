pub struct BlogPost {
    pub title: String,
    pub content: String,
    pub slug: String,
}

impl BlogPost {
    pub fn new(slug: &str, title: &str, content: &str) -> BlogPost {
        Self {
            title: title.to_string(),
            content: content.to_string(),
            slug: slug.to_string(),
        }
    }
}
