mod api;
mod db;
mod errors;
mod memory;
mod mime;
mod plugin;
mod post;
mod plugin_manager;
mod render;
mod routes;

use std::env;
use std::sync::{Arc, Mutex};

use actix_web::{App, HttpServer, web::Data};
use plugin_manager::PluginManager;
use rusqlite::Connection;

use crate::memory::get_process_memory;
use crate::post::BlogPost;

const POSTS_SEED: &str = include_str!("../assets/posts-seed.json");

#[derive(Clone)]
pub struct AppState {
    pub conn: Arc<Mutex<Connection>>,
    pub plugin_manager: Arc<Mutex<PluginManager>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let num_cpus = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
    let mut host = "0.0.0.0".to_string();
    let mut port = 8080;
    let mut num_workers = num_cpus;
    let mut debug = false;

    let args: Vec<String> = env::args().collect();
    let  mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--host" => {
                if i + 1 < args.len() {
                    host = args[i + 1].clone();
                    i += 1;
                }
            }
            "--port" => {
                if i + 1 < args.len() {
                    if let Ok(p) = args[i + 1].parse::<u16>() {
                        port = p;
                        i += 1;
                    }
                }
            }
            "--num_workers" => {
                if i + 1 < args.len() {
                    if let Ok(n) = args[i + 1].parse::<usize>() {
                        num_workers = n;
                        i += 1;
                    }
                }
            }
            "--debug" => {
                debug = true;
            }
            _ => {}
        }
        i += 1;
    }

    if debug {
        unsafe {std::env::set_var("RUST_LOG", "debug");}
        env_logger::init();
    }

    let m1 = get_process_memory();
    // Initializing DB for blog posts.
    let m3 = get_process_memory();
    // Run a server.use post::BlogPost;
    //server::run_server(conn)
    let conn = Connection::open("pagezest.db").expect("Could not open DB");
    db::init_db(&conn).expect("Could not init DB");
    let m2 = get_process_memory();
    // If no blogs are there then create one sample blog.
    if db::get_all_post(&conn).unwrap().is_empty() {
        let blog_posts: Vec<BlogPost> = serde_json::from_str(POSTS_SEED).unwrap();
        for blog_post in blog_posts {
            db::create_post(&conn, blog_post).unwrap();
        }
    }

    println!("Starting Pagezest Instance");
    println!("Initial Memory at : {} KB", m1);
    println!("DB Initialized Memory : {} KB", m2);
    println!("Sample Post Generated : {} KB", m3);

    let mut plugin_manager = PluginManager::new();
    plugin_manager.scan_plugins().unwrap();

    let conn = Arc::new(Mutex::new(conn));
    let plugin_manager = Arc::new(Mutex::new(plugin_manager));
    HttpServer::new(move || {
        let data = AppState {
            conn: conn.clone(),
            plugin_manager: plugin_manager.clone(),
        };
        let data = Data::new(data);
        App::new()
            .app_data(data)
            .configure(routes::config)
    })
    .workers(num_workers)
        .bind((host, port))?
        .run()
        .await
}
