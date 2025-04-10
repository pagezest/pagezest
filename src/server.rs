use std::sync::{ Arc, Mutex };

use rusqlite::Connection;
use tiny_http::{ Response, Server };

use crate::api::route_request;
use crate::errors::AppError;

pub fn run_server(conn: Connection) -> Result<(), AppError> {
    let shared_conn = Arc::new(Mutex::new(conn));
    let server = Server::http("0.0.0.0:8080")?;

    for request in server.incoming_requests() {
        let path = request.url().to_string();
        let method = request.method().clone(); // clone to extend lifetime
        let mut request = request; // shadowing as mutable after prior borrows

        let response = match route_request(&method, &path, &shared_conn, &mut request) {
            Ok(body) => {
                let body_str = serde_json
                    ::to_string(&body)
                    .unwrap_or_else(|_| "{\"error\": \"Internal Server Error\"}".to_string());
                Response::from_string(body_str).with_header(
                    tiny_http::Header
                        ::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                        .unwrap()
                )
            }
            Err(e) => {
                let error_msg = format!("{{\"error\": \"{}\"}}", e);
                Response::from_string(error_msg)
                    .with_status_code(500)
                    .with_header(
                        tiny_http::Header
                            ::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                            .unwrap()
                    )
            }
        };

        let _ = request.respond(response);
    }

    Ok(())
}
