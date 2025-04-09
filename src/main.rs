// Blog Posts
// Create Post
// Read Post
// Read All Posts
mod db;
mod errors;
mod post;
mod server;

use post::BlogPost;
use rusqlite::Connection;

use crate::errors::AppError;

fn main() -> Result<(), AppError> {
    // Initializing DB for blog posts.
    let conn = Connection::open("pagezest.db")?;
    db::init_db(&conn)?;

    // If no blogs are there then create one sample blog.
    if db::get_all_post(&conn)?.is_empty() {
        let blog_post = BlogPost::new("example", "PageZestExample Blog", "<h1> Example </h1>");
        db::create_post(&conn, blog_post)?;
    }

    // Run a server.
    server::run_server(conn)
}
