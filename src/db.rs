use std::sync::Arc;

use crate::{inmemory_cache::ShardedCache, post::BlogPost};
use rusqlite::Result;
use sqlx::{pool::PoolOptions, query, sqlite::SqliteQueryResult, Error, Pool, Row, Sqlite};

pub type DBPool = Pool<Sqlite>;
pub type DBPoolOptions = PoolOptions<Sqlite>;
pub type DBQueryResult = SqliteQueryResult;

pub async fn init_db(conn: &DBPool) -> Result<DBQueryResult, Error> {
    let table_creation = r#"
    CREATE TABLE IF NOT EXISTS posts (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        title TEXT NOT NULL,
        content TEXT NOT NULL,
        content_flatbuffer BLOB NOT NULL,
        content_cached BLOB NULL,
        slug TEXT NOT NULL UNIQUE,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    )
    "#;
    query(table_creation).execute(&*conn).await
}

pub async fn create_post(conn: &DBPool, blog_post: BlogPost) -> Result<DBQueryResult, Error> {
    let create_post_query = r#"
        INSERT INTO posts(slug, title, content, content_flatbuffer, content_cached)
        VALUES (?, ?, ?, ?, ?)
        ON CONFLICT(slug) DO NOTHING
    "#;
    let content_str = blog_post.content.to_string();
    query(create_post_query)
        .bind(blog_post.slug)
        .bind(blog_post.title)
        .bind(content_str)
        .bind(blog_post.content_flatbuffer)
        .bind(blog_post.content_cached)
        .execute(&*conn)
        .await
}

/*
*/
pub async fn get_all_post(conn: &DBPool) -> Result<Vec<BlogPost>, Error> {
    let fetch_all_post_query = r#"
        SELECT id, slug, title, created_at, updated_at
        FROM posts
        ORDER BY created_at DESC
    "#;

    let rows = query(fetch_all_post_query).fetch_all(&*conn).await?;

    Ok(rows
        .iter()
        .map(|row| BlogPost {
            id: row.get("id"),
            slug: row.get("slug"),
            title: row.get("title"),
            content: "".to_string(),
            content_flatbuffer: vec![],
            content_cached: vec![],
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
        .collect())
}

pub async fn get_post_by_id(conn: &DBPool, id: &str) -> Result<Option<BlogPost>, Error> {
    let qry = r#"
        SELECT id, slug, title, content, created_at, updated_at
        FROM posts WHERE id = ?1
        "#;

    let row = query(qry).bind(id).fetch_one(&*conn).await?;

    Ok(Some(BlogPost::new_with_id(
        row.get("id"),
        row.get("slug"),
        row.get("title"),
        row.get::<&str, &str>("content"),
        vec![],
        //row.get("content_flatbuffer"),
    )))
}

pub async fn get_post_by_slug_cached(
    conn: &DBPool,
    cache: &Arc<ShardedCache<BlogPost>>,
    slug: &str,
) -> Result<Option<BlogPost>, Error> {
    if cache.has(slug) {
        return Ok(cache.get(slug));
    }
    let post = get_post_by_slug(conn, slug).await?;

    match post {
        Some(post) => {
            cache.set(slug.to_string(), post.clone());
            Ok(Some(post))
        }
        _ => Ok(None),
    }
}

pub async fn get_post_by_slug(conn: &DBPool, slug: &str) -> Result<Option<BlogPost>, Error> {
    let qry = r#"
        SELECT id, slug, title, content_flatbuffer, content_cached, created_at, updated_at
        FROM posts WHERE slug = ?1
        "#;

    let row = query(qry).bind(slug).fetch_one(&*conn).await?;

    Ok(Some(BlogPost {
        id: row.get("id"),
        slug: row.get("slug"),
        title: row.get("title"),
        content: "".to_string(),
        content_cached: row.get("content_cached"),
        content_flatbuffer: row.get("content_flatbuffer"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }))
}

pub async fn update_post(
    conn: &DBPool,
    blog_post: BlogPost,
    id: &str,
) -> Result<DBQueryResult, Error> {
    let update_post_query = r#"
        UPDATE posts
        SET title = ?, content = ?, content_flatbuffer = ?, content_cached = ?, slug = ?, updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#;

    let content_str = blog_post.content.to_string();
    query(update_post_query)
        .bind(blog_post.title)
        .bind(content_str)
        .bind(blog_post.content_flatbuffer)
        .bind(blog_post.content_cached)
        .bind(blog_post.slug.clone())
        .bind(id.to_string())
        .execute(&*conn)
        .await
}

pub async fn delete_post(conn: &DBPool, id: &str, by_slug: bool) -> Result<DBQueryResult, Error> {
    let delete_post_query = if by_slug {
        r#"
        DELETE FROM posts WHERE slug = ?
        "#
    } else {
        r#"
        DELETE FROM posts WHERE id = ?
        "#
    };
    query(delete_post_query).bind(id).execute(&*conn).await
}
/*

pub fn get_stats(conn: &DBPool) -> Result<Value> {
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
*/
