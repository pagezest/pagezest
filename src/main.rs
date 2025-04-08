// Blog Posts
// Create Post
// Read Post
// Read All Posts
mod errors;
mod server;
mod db;
mod post;

use post::BlogPost;
use rusqlite::Connection;

use crate::errors::AppError;

fn main() -> Result<(), AppError> {
    // Initializing DB for blog posts.
    let conn = Connection::open("pagezest.db")?;
    db::init_db(&conn)?;

    // TODO : If no blogs are there then create one sample blog.
    let blog_post = BlogPost::new("example", "PageZestExample Blog", "<h1> Example </h1>");
    db::create_post(&conn, blog_post)?;

    // Run a server.
    server::run_server(conn)
}
