mod api;
mod db;
mod errors;
mod inmemory_cache;
mod memory;
mod plugin;
mod plugin_manager;
mod post;
#[allow(warnings, dead_code, unused_imports)]
#[path = "./post_flatbuffers.rs"]
mod post_flatbuffers;
mod render_flatbuffers;
mod routes;

use std::sync::Arc;
use std::{env, sync::RwLock};

use crate::memory::get_process_memory;
use actix::{Actor, Addr};
use actix_web::{web::Data, App, HttpServer};
use db::{DBPool, DBPoolOptions};
use inmemory_cache::ShardedCache;
use plugin_manager::PluginManager;
use post::BlogPost;

#[derive(Clone)]
pub struct AppState {
    pub conn: Arc<DBPool>,
    pub cache: Arc<Addr<ShardedCache<BlogPost>>>,
    pub plugin_manager: Arc<RwLock<PluginManager>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let num_cpus = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    let mut host = "0.0.0.0".to_string();
    let mut port = 8080;
    let mut num_workers = num_cpus;
    let mut db_pool_size = num_cpus;
    let mut db_cache_size = 1024;
    let mut debug = false;

    let args: Vec<String> = env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--host" | "-h" => {
                if i + 1 < args.len() {
                    host = args[i + 1].clone();
                    i += 1;
                }
            }
            "--port" | "-p" => {
                if i + 1 < args.len() {
                    if let Ok(p) = args[i + 1].parse::<u16>() {
                        port = p;
                        i += 1;
                    }
                }
            }
            "--num_workers" | "-w" => {
                if i + 1 < args.len() {
                    if let Ok(n) = args[i + 1].parse::<usize>() {
                        num_workers = n;
                        i += 1;
                    }
                }
            }
            "--pool_size" | "-c" => {
                if i + 1 < args.len() {
                    if let Ok(n) = args[i + 1].parse::<usize>() {
                        db_pool_size = n;
                        i += 1;
                    }
                }
            }
            "--cache_size" | "-z" => {
                if i + 1 < args.len() {
                    if let Ok(n) = args[i + 1].parse::<usize>() {
                        db_cache_size = n;
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

    println!(
        "DB options:\n\tworkers: {}\n\tpool_size: {}\n\tcache_size: {}",
        num_workers, db_pool_size, db_cache_size
    );

    if debug {
        unsafe {
            std::env::set_var("RUST_LOG", "debug");
        }
        env_logger::init();
    }

    let m1 = get_process_memory();
    // Initializing DB for blog posts.
    let m3 = get_process_memory();
    // Run a server.use post::BlogPost;
    //server::run_server(conn)
    let mut db_path = env::current_dir().unwrap();
    db_path.push("pagezest.db");

    // workaround for DB file not created automatically, error 14
    {
        std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(format!("{}", db_path.display()))
            .expect("failed to touch db.db");
    }

    let database_url = format!("sqlite://{}", db_path.display());

    let pool = DBPoolOptions::new()
        .min_connections(0)
        .max_connections(db_pool_size as u32)
        .connect(&database_url)
        .await
        .expect("could not open DB");
    db::init_db(&pool).await.expect("could not init DB");

    for _ in 0..(db_pool_size) {
        let mut conn = pool.acquire().await.expect("could not acquire connection");
        sqlx::query("PRAGMA journal_mode=WAL")
            .execute(&mut *conn)
            .await
            .expect("could not execute journal_mode=WAL");
        sqlx::query(&format!("PRAGMA cache_size = {};", db_cache_size))
            .execute(&mut *conn)
            .await
            .expect("could not execute cache_size = 1024");
    }
    let m2 = get_process_memory();
    // If no blogs are there then create one sample blog.
    /*
        if db::get_all_post(&pool).unwrap().is_empty() {
            let blog_posts: Vec<BlogPost> = serde_json::from_str(POSTS_SEED).unwrap();
            for blog_post in blog_posts {
                //db::create_post(&pool, blog_post).unwrap();
            }
        }
    */
    println!("Starting Pagezest Instance");
    println!("Initial Memory at : {} KB", m1);
    println!("DB Initialized Memory : {} KB", m2);
    println!("Sample Post Generated : {} KB", m3);

    let pool = Arc::new(pool);
    HttpServer::new(move || {
        let mut plugin_manager = PluginManager::new();
        plugin_manager.scan_plugins().unwrap();

        let plugin_manager = Arc::new(RwLock::new(plugin_manager));
        let cache: Addr<ShardedCache<BlogPost>> = ShardedCache::new(1).start();
        let data = AppState {
            conn: pool.clone(),
            cache: Arc::new(cache),
            plugin_manager: plugin_manager.clone(),
        };
        let data = Data::new(data);
        App::new().app_data(data).configure(routes::config)
    })
    .workers(num_workers)
    .bind((host, port))?
    .run()
    .await
}
