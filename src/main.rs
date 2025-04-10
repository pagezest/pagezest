// Blog Posts
// Create Post
// Read Post
// Read All Posts
mod api;
mod db;
mod errors;
mod post;
mod server;

use rusqlite::Connection;
use serde_json::json;
use crate::post::BlogPost;

use crate::errors::AppError;

fn main() -> Result<(), AppError> {
    // Initializing DB for blog posts.
    let conn = Connection::open("pagezest.db")?;
    db::init_db(&conn)?;

    // If no blogs are there then create one sample blog.
    if db::get_all_post(&conn)?.is_empty() {
        let blog_post = BlogPost::new(
            "example",
            "PageZestExample Blog",
            json!({"title" : "Pagezest"})
        );
        db::create_post(&conn, blog_post).unwrap();
    }

    // Run a server.use post::BlogPost;
    server::run_server(conn)
}
