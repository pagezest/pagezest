// Blog Posts
// Create Post
// Read Post
// Read All Posts
mod api;
mod db;
mod errors;
mod memory;
mod mime;
mod plugin;
mod post;
mod server;
mod plugin_manager;
mod render;

use rusqlite::Connection;
use serde_json::json;

use crate::errors::AppError;
use crate::memory::get_process_memory;
use crate::post::BlogPost;

fn main() -> Result<(), AppError> {
    let m1 = get_process_memory();
    // Initializing DB for blog posts.
    let conn = Connection::open("pagezest.db")?;
    db::init_db(&conn)?;
    let m2 = get_process_memory();
    // If no blogs are there then create one sample blog.
    if db::get_all_post(&conn)?.is_empty() {
        let blog_post = BlogPost::new(
            "",
            "PageZest Example Blog",
            json!({
                "json":[{"depth":1,"raw":"# This is heading1\n","text":"This is heading1","tokens":[{"escaped":false,"raw":"This is heading1","text":"This is heading1","type":"text"}],"type":"heading"},{"raw":"Contents of Line1\n","text":"Contents of Line1","tokens":[{"escaped":false,"raw":"Contents of Line1","text":"Contents of Line1","type":"text"}],"type":"paragraph"},{"depth":2,"raw":"##    This is Heading2\n","text":"This is Heading2","tokens":[{"escaped":false,"raw":"This is Heading2","text":"This is Heading2","type":"text"}],"type":"heading"},{"raw":"Contents of Line2","text":"Contents of Line2","tokens":[{"escaped":false,"raw":"Contents of Line2","text":"Contents of Line2","type":"text"}],"type":"paragraph"}],
                "md":"# This is heading1\nContents of Line1\n##    This is Heading2\nContents of Line2"}
            ),
        );
        db::create_post(&conn, blog_post).unwrap();
    }
    let m3 = get_process_memory();
    println!("Starting Pagezest Instance");
    println!("Initial Memory at : {} KB", m1);
    println!("DB Initialized Memory : {} KB", m2);
    println!("Sample Post Generated : {} KB", m3);
    // Run a server.use post::BlogPost;
    server::run_server(conn)
}
