use rusqlite::Connection;
use serde_json::{ Value, json };
use std::{ io::Cursor, sync::{ Arc, Mutex } };
use tiny_http::{ Header, Method, Request, Response };

use crate::{ db, errors::AppError, post::BlogPost, plugin::call_wasm };

pub fn route_request(
    method: &Method,
    path: &str,
    conn: &Arc<Mutex<Connection>>,
    request: &mut Request
) -> Result<Value, AppError> {
    match (method, path) {
        (&Method::Get, "/") => health_check(),
        (&Method::Get, p) if p.starts_with("/api/blog/") => find_blog_by_slug(conn, p),
        (&Method::Get, "/api/blogs") => get_all_blog_posts(conn),
        (&Method::Post, "/api/blog/new") => create_new_blog_post(conn, request),
        (&Method::Post, "/api/blog/update") => update_blog_post(conn, request),
        (&Method::Post, "/api/plugin/demo") => get_table_of_contents(request),
        (&Method::Delete, p) if p.starts_with("/api/blog/delete/") =>
            delete_blog_post(conn, request),
        _ => Err(AppError::PageNotFound(path.to_string())),
    }
}

fn health_check() -> Result<Value, AppError> {
    Ok(json!({"msg": "Server is healthy", "success": true}))
}

fn find_blog_by_slug(conn: &Arc<Mutex<Connection>>, slug: &str) -> Result<Value, AppError> {
    let slug = slug.strip_prefix("/api/blog/").unwrap();
    let post = db::get_post_by_slug(&conn.lock().unwrap(), slug)?;
    match post {
        Some(post) => Ok(json!({"data" : post, "sucess": true})),
        None => Err(AppError::PageNotFound(format!("No post found for slug: {}", slug))),
    }
}

fn get_all_blog_posts(conn: &Arc<Mutex<Connection>>) -> Result<Value, AppError> {
    let posts = db::get_all_post(&conn.lock().unwrap())?;
    Ok(json!({"data" : posts, "success": true}))
}

fn create_new_blog_post(
    conn: &Arc<Mutex<Connection>>,
    request: &mut Request
) -> Result<Value, AppError> {
    let mut req_body = String::new();
    request.as_reader().read_to_string(&mut req_body).unwrap();

    // Validate request and convert to BlogPost Model.
    let blog_post: Result<BlogPost, serde_json::Error> = serde_json::from_str(&req_body);
    match blog_post {
        Ok(blog) => {
            // Check if Blog with given slug already exists or not.
            if db::get_post_by_slug(&conn.lock().unwrap(), &blog.slug)?.is_some() {
                return Ok(
                    json!({"msg": format!("Blog with slug {} already exists.", blog.slug), "success": false})
                );
            }
            db::create_post(&conn.lock().unwrap(), blog)?;
            Ok(json!({"msg" : "New Blog Created Successfully", "success": true}))
        }
        Err(e) => { Ok(json!({"msg" : format!("Failed to Create blog {}", e), "success" : false})) }
    }
}

fn update_blog_post(
    conn: &Arc<Mutex<Connection>>,
    request: &mut Request
) -> Result<Value, AppError> {
    let mut req_body = String::new();
    request.as_reader().read_to_string(&mut req_body).unwrap();

    // Validate request and convert to BlogPost Model.
    let blog_post: Result<BlogPost, serde_json::Error> = serde_json::from_str(&req_body);
    match blog_post {
        Ok(blog) => {
            // Check if Blog with given slug already exists or not.
            if db::get_post_by_slug(&conn.lock().unwrap(), &blog.slug)?.is_some() {
                db::update_post(&conn.lock().unwrap(), blog).unwrap();
                return Ok(json!({"msg" : "Updated your Blog Successfully", "success" : true}));
            }
            Ok(
                json!({"msg": format!("No blog found with slug - {}", blog.slug), "success" : false})
            )
        }
        Err(e) => { Ok(json!({"msg" : format!("Failed to Update blog {}", e), "success" : false})) }
    }
}

fn delete_blog_post(
    conn: &Arc<Mutex<Connection>>,
    request: &mut Request
) -> Result<Value, AppError> {
    let mut req_body = String::new();
    request.as_reader().read_to_string(&mut req_body).unwrap();
    let blog_post: Result<BlogPost, serde_json::Error> = serde_json::from_str(&req_body);
    match blog_post {
        Ok(blog) => {
            // Check if Blog with given slug already exists or not.
            if db::get_post_by_slug(&conn.lock().unwrap(), &blog.slug)?.is_some() {
                db::delete_post(&conn.lock().unwrap(), &blog.slug)?;
                return Ok(json!({"msg": "Blog deleted successfully", "success" : true}));
            }
            Ok(
                json!({"msg": format!("No blog found with slug - {}", blog.slug), "success" : false})
            )
        }
        Err(e) => { Ok(json!({"msg" : format!("Failed to Delete blog {}", e), "success" : false})) }
    }
}

pub fn load_sample_image() -> Result<Response<Cursor<Vec<u8>>>, AppError> {
    let data = std::fs::read("sample.jpg")?;
    let mut response = Response::from_data(data);
    response.add_header(Header::from_bytes(&b"Content-Type"[..], &b"image/jpeg"[..]).unwrap());
    Ok(response)
}

pub fn get_table_of_contents(request: &mut Request) -> Result<Value, AppError> {
    let mut req_body = String::new();
    request.as_reader().read_to_string(&mut req_body).unwrap();

    let req_json: Value = serde_json
        ::from_str(&req_body)
        .map_err(|e| AppError::ServerError(e.to_string()))?;
    let md_content = req_json
        .get("content")
        .and_then(|c| c.as_str())
        .ok_or_else(|| {
            AppError::ServerError("Missing or invalid 'content' field in request body".to_string())
        })?;

    let toc_raw = call_wasm("plugins/toc.wasm", md_content, "toc").unwrap();
    let toc_lines: Vec<&str> = toc_raw.split('\n').collect();
    let toc: Vec<Value> = toc_lines
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let level = line
                .chars()
                .take_while(|&c| c == '\t')
                .count();
            json!({"level": level+1, "heading": line.trim()})
        })
        .collect();

    Ok(json!({"data": toc, "success": true}))
}
