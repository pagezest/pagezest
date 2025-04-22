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
            "example",
            "PageZest Example Blog",
            json!(
                {
                    "md" : "#This is heading1\nContents of Line1\n##    This is Heading2\n  Contents of Line2",
                    "json": {
                    "type": "root",
                    "children": [
                        {
                            "type": "heading",
                            "raw": "# Heading-1\n\n",
                            "depth": 1,
                            "text": "Heading-1",
                            "tokens": [
                                {
                                    "type": "text",
                                    "raw": "Heading-1",
                                    "text": "Heading-1",
                                    "escaped": false
                                }
                            ]
                        },
                        {
                            "type": "heading",
                            "raw": "## Heading-2\n\n",
                            "depth": 2,
                            "text": "Heading-2",
                            "tokens": [
                                {
                                    "type": "text",
                                    "raw": "Heading-2",
                                    "text": "Heading-2",
                                    "escaped": false
                                }
                            ]
                        }
                    ]
                }
                }
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
