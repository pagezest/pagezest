use std::io::Cursor;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use tiny_http::{Response, Server};

use crate::api::{ResponseType, load_sample_image, route_request};
use crate::errors::AppError;
use crate::memory::get_process_memory;
use crate::plugin_manager::PluginManager;

pub fn run_server(conn: Connection) -> Result<(), AppError> {
    println!("Starting Web Server");
    let m1 = get_process_memory();
    let shared_conn = Arc::new(Mutex::new(conn));
    let mut plugin_manager = PluginManager::new();
    plugin_manager.scan_plugins().expect("Plugins scan error");
    let shared_plugin_manager = Arc::new(Mutex::new(plugin_manager));
    let server = Server::http("0.0.0.0:8080")?;
    println!("Memory after starting server : {} KB", m1);

    let mut prev_request_memory = get_process_memory();
    for request in server.incoming_requests() {
        let path = request.url().to_string();
        let method = request.method().clone(); // clone to extend lifetime
        let mut request = request; // shadowing as mutable after prior borrows

        let response: Response<Cursor<Vec<u8>>>;
        if path == "/api/test-image" {
            response = load_sample_image().unwrap();
        } else {
            response = match route_request(&method, &path, &shared_conn, &shared_plugin_manager, &mut request) {
                Ok(ResponseType::Json(body)) => {
                    let body_str = serde_json::to_string(&body)
                        .unwrap_or_else(|_| "{\"error\": \"Internal Server Error\"}".to_string());
                    Response::from_string(body_str).with_header(
                        tiny_http::Header::from_bytes(
                            &b"Content-Type"[..],
                            &b"application/json"[..],
                        )
                        .unwrap(),
                    )
                }
                Ok(ResponseType::Html(body)) => Response::from_string(body).with_header(
                    tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap(),
                ),
                Ok(ResponseType::Binary(body, mime)) => Response::from_data(body).with_header(
                    tiny_http::Header::from_bytes(&b"Content-Type"[..], &mime[..]).unwrap(),
                ),
                Err(e) => {
                    let error_msg = format!("{{\"error\": \"{}\"}}", e);
                    Response::from_string(error_msg)
                        .with_status_code(500)
                        .with_header(
                            tiny_http::Header::from_bytes(
                                &b"Content-Type"[..],
                                &b"application/json"[..],
                            )
                            .unwrap(),
                        )
                }
            };
        }

        let current_request_memory = get_process_memory();
        let _ = request.respond(response);
        let overhead = current_request_memory.saturating_sub(prev_request_memory);
        if overhead > 0 {
            println!("Memory Overhead per request : {} KB ", overhead);
        }
        prev_request_memory = current_request_memory;
    }

    Ok(())
}
