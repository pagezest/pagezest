use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use tiny_http::{Response, Server};

use crate::db::get_post_by_slug;
use crate::errors::AppError;

pub fn run_server(conn: Connection) -> Result<(), AppError> {
    let shared_conn = Arc::new(Mutex::new(conn));
    let server = Server::http("0.0.0.0:8080")?;

    for request in server.incoming_requests() {
        let path = request.url().to_string();
        let response = match route_request(&path, &shared_conn) {
            Ok(body) => Response::from_string(body),
            Err(e) => {
                let error_msg = format!("Error: {}", e);
                Response::from_string(error_msg).with_status_code(500)
            }
        };

        request.respond(response)?;
    }
    Ok(())
}

fn route_request(path: &str, conn: &Arc<Mutex<Connection>>) -> Result<String, AppError> {
    if path == "/" {
        // HomePage logic will come here.
        return Ok("<h1>Pagezest Home</h1>".to_string());
    } else if let Some(slug) = path.strip_prefix("/blog/") {
        // This will handle /blog/abc-123
        let post = get_post_by_slug(&conn.lock().unwrap(), slug)?;
        match post {
            Some(post) => Ok(post.content),
            None => Err(AppError::PageNotFound(format!(
                "No Post Found for slug : {}",
                slug
            ))),
        }
    } else {
        // 404
        Err(AppError::PageNotFound(format!(
            "Route Not Allowed {}",
            path
        )))
    }
}
