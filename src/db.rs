use crate::post::BlogPost;
use rusqlite::{Connection, Result};

pub fn init_db(conn: &Connection) -> Result<()> {
    let table_creation = r#"
    CREATE TABLE IF NOT EXISTS posts (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        title TEXT NOT NULL,
        content TEXT NOT NULL,
        slug TEXT NOT NULL,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    )
    "#;
    conn.execute(table_creation, [])?;
    Ok(())
}

pub fn create_post(conn: &Connection, blog_post: BlogPost) -> Result<()> {
    let create_post_query = "INSERT INTO posts(slug, title, content) VALUES (?, ?, ?)";
    let params = [blog_post.slug, blog_post.title, blog_post.content];
    conn.execute(create_post_query, params)?;
    Ok(())
}

pub fn get_all_post(conn: &Connection) -> Result<Vec<BlogPost>> {
    let fetch_all_post_query = r#"
            SELECT slug, title, content
            FROM posts
        "#;
    let mut stmt = conn.prepare(fetch_all_post_query)?;
    let rows = stmt.query_map([], |row| {
        Ok(BlogPost {
            slug: row.get(0)?,
            title: row.get(1)?,
            content: row.get(2)?,
        })
    })?;

    let mut posts = Vec::new();
    for row in rows {
        posts.push(row?);
    }
    Ok(posts)
}

pub fn get_post_by_slug(conn: &Connection, slug: &str) -> Result<Option<BlogPost>> {
    let fetch_post_by_slug_query = r#"
            SELECT slug, title, content
            FROM posts WHERE slug = ?1
        "#;
    let mut stmt = conn.prepare(fetch_post_by_slug_query)?;

    let mut rows = stmt.query_map([slug], |row| {
        Ok(BlogPost {
            slug: row.get(0)?,
            title: row.get(1)?,
            content: row.get(2)?,
        })
    })?;
    if let Some(row) = rows.next() {
        Ok(Some(row?))
    } else {
        Ok(None)
    }
}
