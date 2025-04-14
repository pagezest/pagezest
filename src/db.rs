use crate::post::BlogPost;
use rusqlite::{ Connection, Result };
use serde_json::Value;

pub fn init_db(conn: &Connection) -> Result<()> {
    let table_creation =
        r#"
    CREATE TABLE IF NOT EXISTS posts (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        title TEXT NOT NULL,
        content TEXT NOT NULL,
        slug TEXT NOT NULL UNIQUE,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    )
    "#;
    conn.execute(table_creation, [])?;
    Ok(())
}

pub fn create_post(conn: &Connection, blog_post: BlogPost) -> Result<()> {
    let create_post_query =
        r#"
        INSERT INTO posts(slug, title, content)
        VALUES (?, ?, ?)
        ON CONFLICT(slug) DO NOTHING
    "#;
    let content_str = blog_post.content.to_string();
    let params = [blog_post.slug, blog_post.title, content_str];
    conn.execute(create_post_query, params)?;
    Ok(())
}

pub fn get_all_post(conn: &Connection) -> Result<Vec<BlogPost>> {
    let fetch_all_post_query = r#"
        SELECT slug, title, content, created_at
        FROM posts
    "#;

    let mut stmt = conn.prepare(fetch_all_post_query)?;
    let rows = stmt.query_map([], |row| {
        let slug: String = row.get(0)?;
        let title: String = row.get(1)?;
        let content_str: String = row.get(2)?;
        let content_json: Value = serde_json::from_str(&content_str).unwrap();
        let created_at: String = row.get(3)?;
        Ok(BlogPost {
            slug,
            title,
            content: content_json,
            created_at,
        })
    })?;

    let mut posts = Vec::new();
    for row in rows {
        posts.push(row?);
    }
    Ok(posts)
}

pub fn get_post_by_slug(conn: &Connection, slug: &str) -> Result<Option<BlogPost>> {
    let fetch_post_by_slug_query =
        r#"
        SELECT slug, title, content, created_at
        FROM posts WHERE slug = ?1
    "#;
    let mut stmt = conn.prepare(fetch_post_by_slug_query)?;

    let mut rows = stmt.query_map([slug], |row| {
        let slug: String = row.get(0)?;
        let title: String = row.get(1)?;
        let content_str: String = row.get(2)?;
        let content_json: Value = serde_json::from_str(&content_str).unwrap();
        let created_at: String = row.get(3)?;
        Ok(BlogPost {
            slug,
            title,
            content: content_json,
            created_at,
        })
    })?;

    if let Some(row) = rows.next() {
        Ok(Some(row?))
    } else {
        Ok(None)
    }
}

pub fn update_post(conn: &Connection, blog_post: BlogPost) -> Result<()> {
    let update_post_query =
        r#"
        UPDATE posts
        SET title = ?, content = ?
        WHERE slug = ?
    "#;
    let content_str = blog_post.content.to_string();
    let params = [blog_post.title, content_str, blog_post.slug];
    conn.execute(update_post_query, params)?;
    Ok(())
}

pub fn delete_post(conn: &Connection, id: &str) -> Result<()> {
    let delete_post_query = r#"
        DELETE FROM posts WHERE id = ?
    "#;
    conn.execute(delete_post_query, [id])?;
    Ok(())
}
