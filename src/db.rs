use rusqlite::{ Connection, Result };
use crate::post::BlogPost;

pub fn init_db(conn: &Connection) -> Result<()> {
    let table_creation =
        r#"
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

// TODO : Get Blog by Slug.
// TODO : Get All Blogs.
