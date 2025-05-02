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

use crate::errors::AppError;
use crate::memory::get_process_memory;
use crate::post::BlogPost;

const POSTS_SEED: &str = include_str!("../assets/posts-seed.json");

fn main() -> Result<(), AppError> {
    let m1 = get_process_memory();
    // Initializing DB for blog posts.
    let conn = Connection::open("pagezest.db")?;
    db::init_db(&conn)?;
    let m2 = get_process_memory();
    // If no blogs are there then create one sample blog.
    if db::get_all_post(&conn)?.is_empty() {
        let blog_posts: Vec<BlogPost> = serde_json::from_str(POSTS_SEED).unwrap();
        for blog_post in blog_posts {
            db::create_post(&conn, blog_post).unwrap();
        }
    }
    let m3 = get_process_memory();
    println!("Starting Pagezest Instance");
    println!("Initial Memory at : {} KB", m1);
    println!("DB Initialized Memory : {} KB", m2);
    println!("Sample Post Generated : {} KB", m3);
    // Run a server.use post::BlogPost;
    server::run_server(conn)
}
