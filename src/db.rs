use crate::post::BlogPost;
use rusqlite::{Connection, Result};
use serde_json::{Value, json};

pub fn init_db(conn: &Connection) -> Result<()> {
    let table_creation = r#"
    CREATE TABLE IF NOT EXISTS posts (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        title TEXT NOT NULL,
        content TEXT NOT NULL,
        slug TEXT NOT NULL UNIQUE,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    )
    "#;
    conn.execute(table_creation, [])?;
    Ok(())
}

pub fn create_post(conn: &Connection, blog_post: BlogPost) -> Result<()> {
    let create_post_query = r#"
        INSERT INTO posts(slug, title, content)
        VALUES (?, ?, ?)
        ON CONFLICT(slug) DO NOTHING
    "#;
    let content_str = blog_post.content.to_string();
    let params = [blog_post.slug, blog_post.title, content_str];
    conn.execute(create_post_query, params)?;
    Ok(())
}

pub fn get_all_post(conn: &Connection) -> Result<Vec<Value>> {
    let fetch_all_post_query = r#"
        SELECT id, slug, title, created_at, updated_at
        FROM posts
        ORDER BY created_at DESC
    "#;

    let mut stmt = conn.prepare(fetch_all_post_query)?;
    let rows = stmt.query_map([], |row| {
        let id: i32 = row.get(0)?;
        let slug: String = row.get(1)?;
        let title: String = row.get(2)?;
        let created_at: String = row.get(3)?;
        let updated_at: String = row.get(4)?;
        Ok(json!({
            "id": id,
            "slug": slug,
            "title": title,
            "created_at": created_at,
            "updated_at": updated_at
        }))
    })?;

    let mut posts = Vec::new();
    for row in rows {
        posts.push(row?);
    }
    Ok(posts)
}

pub fn get_post(conn: &Connection, identifier: &str, by_slug: bool) -> Result<Option<BlogPost>> {
    let query = if by_slug {
        r#"
        SELECT slug, title, content, created_at, updated_at
        FROM posts WHERE slug = ?1
        "#
    } else {
        r#"
        SELECT slug, title, content, created_at, updated_at
        FROM posts WHERE id = ?1
        "#
    };

    let mut stmt = conn.prepare(query)?;

    let mut rows = stmt.query_map([identifier], |row| {
        let slug: String = row.get(0)?;
        let title: String = row.get(1)?;
        let content_str: String = row.get(2)?;
        let content_json: Value = serde_json::from_str(&content_str).unwrap();
        let created_at: String = row.get(3)?;
        let updated_at: String = row.get(4)?;
        Ok(BlogPost {
            slug,
            title,
            content: content_json,
            created_at,
            updated_at,
        })
    })?;

    if let Some(row) = rows.next() {
        Ok(Some(row?))
    } else {
        Ok(None)
    }
}

pub fn update_post(
    conn: &Connection,
    blog_post: BlogPost,
    identifier: &str,
    by_slug: bool,
) -> Result<()> {
    let update_post_query = if by_slug {
        r#"
        UPDATE posts
        SET title = ?, content = ?, slug = ?, updated_at = CURRENT_TIMESTAMP
        WHERE slug = ?
        "#
    } else {
        r#"
        UPDATE posts
        SET title = ?, content = ?, slug = ?, updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#
    };

    let content_str = blog_post.content.to_string();
    let params: Vec<&dyn rusqlite::ToSql> = if by_slug {
        vec![
            &blog_post.title,
            &content_str,
            &blog_post.slug,
            &blog_post.slug,
        ]
    } else {
        vec![&blog_post.title, &content_str, &blog_post.slug, &identifier]
    };
    conn.execute(update_post_query, &*params)?;
    Ok(())
}

pub fn delete_post(conn: &Connection, identifier: &str, by_slug: bool) -> Result<()> {
    let delete_post_query = if by_slug {
        r#"
        DELETE FROM posts WHERE slug = ?
        "#
    } else {
        r#"
        DELETE FROM posts WHERE id = ?
        "#
    };
    conn.execute(delete_post_query, [identifier])?;
    Ok(())
}

pub fn get_stats(conn: &Connection) -> Result<Value> {
    let fetch_all_post_query = r#"
        SELECT COUNT(*) AS num_posts
        FROM posts
    "#;

    let mut stmt = conn.prepare(fetch_all_post_query)?;
    stmt.query_row([], |row| {
        let num_posts: i32 = row.get(0)?;
        Ok(json!({
            "num_posts": num_posts,
        }))
    })
}
