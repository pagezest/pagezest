use rusqlite::Connection;
use serde_json::{ Value, json };
use std::sync::{ Arc, Mutex };
use tiny_http::{ Method, Request };

use crate::{ db, errors::AppError, plugin::run_plugin, post::BlogPost };

pub enum ResponseType {
    Json(Value),
    Html(String),
}

pub fn route_request(
    method: &Method,
    path: &str,
    conn: &Arc<Mutex<Connection>>,
    request: &mut Request
) -> Result<ResponseType, AppError> {
    match (method, path) {
        (&Method::Get, "/") => home(conn),
        (&Method::Get, "/health-check") => health_check(),
        (&Method::Get, p) if p.starts_with("/api/blog/") => find_blog_by_slug(conn, p),
        (&Method::Get, "/api/blogs") => get_all_blog_posts(conn),
        (&Method::Post, "/api/blog/new") => create_new_blog_post(conn, request),
        (&Method::Post, "/api/blog/update") => update_blog_post(conn, request),
        (&Method::Delete, p) if p.starts_with("/api/blog/delete/") =>
            delete_blog_post(conn, request),
        _ => Err(AppError::PageNotFound(path.to_string())),
    }
}

fn health_check() -> Result<ResponseType, AppError> {
    Ok(ResponseType::Json(json!({"msg": "Server is healthy", "success": true})))
}

fn home(conn: &Arc<Mutex<Connection>>) -> Result<ResponseType, AppError> {
    let posts = json!(db::get_all_post(&conn.lock().unwrap())?);
    let posts_str = posts.to_string();
    let html = run_plugin("homepage.wasm", posts_str)?;
    Ok(ResponseType::Html(html))
}

fn find_blog_by_slug(conn: &Arc<Mutex<Connection>>, slug: &str) -> Result<ResponseType, AppError> {
    let slug = slug.strip_prefix("/api/blog/").unwrap();
    let post = db::get_post_by_slug(&conn.lock().unwrap(), slug)?;
    match post {
        Some(post) => Ok(ResponseType::Json(json!({"data" : post, "sucess": true}))),
        None => Err(AppError::PageNotFound(format!("No post found for slug: {}", slug))),
    }
}

fn get_all_blog_posts(conn: &Arc<Mutex<Connection>>) -> Result<ResponseType, AppError> {
    let posts = db::get_all_post(&conn.lock().unwrap())?;
    Ok(ResponseType::Json(json!({"data" : posts, "success": true})))
}

fn create_new_blog_post(
    conn: &Arc<Mutex<Connection>>,
    request: &mut Request
) -> Result<ResponseType, AppError> {
    let mut req_body = String::new();
    request.as_reader().read_to_string(&mut req_body).unwrap();

    // Validate request and convert to BlogPost Model.
    let blog_post: Result<BlogPost, serde_json::Error> = serde_json::from_str(&req_body);
    match blog_post {
        Ok(blog) => {
            // Check if Blog with given slug already exists or not.
            if db::get_post_by_slug(&conn.lock().unwrap(), &blog.slug)?.is_some() {
                return Ok(
                    ResponseType::Json(json!({"msg": format!("Blog with slug {} already exists.", blog.slug), "success": false})
                    )
                );
            }
            db::create_post(&conn.lock().unwrap(), blog)?;
            Ok(ResponseType::Json(json!({"msg" : "New Blog Created Successfully", "success": true})))
        }
        Err(e) => { Ok(ResponseType::Json(json!({"msg" : format!("Failed to Create blog {}", e), "success" : false}))) }
    }
}

fn update_blog_post(
    conn: &Arc<Mutex<Connection>>,
    request: &mut Request
) -> Result<ResponseType, AppError> {
    let mut req_body = String::new();
    request.as_reader().read_to_string(&mut req_body).unwrap();

    // Validate request and convert to BlogPost Model.
    let blog_post: Result<BlogPost, serde_json::Error> = serde_json::from_str(&req_body);
    match blog_post {
        Ok(blog) => {
            // Check if Blog with given slug already exists or not.
            if db::get_post_by_slug(&conn.lock().unwrap(), &blog.slug)?.is_some() {
                db::update_post(&conn.lock().unwrap(), blog).unwrap();
                return Ok(ResponseType::Json(json!({"msg" : "Updated your Blog Successfully", "success" : true})));
            }
            Ok(
                ResponseType::Json(json!({"msg": format!("No blog found with slug - {}", blog.slug), "success" : false}))
            )
        }
        Err(e) => { Ok(ResponseType::Json(json!({"msg" : format!("Failed to Update blog {}", e), "success" : false}))) }
    }
}

fn delete_blog_post(
    conn: &Arc<Mutex<Connection>>,
    request: &mut Request
) -> Result<ResponseType, AppError> {
    let mut req_body = String::new();
    request.as_reader().read_to_string(&mut req_body).unwrap();
    let blog_post: Result<BlogPost, serde_json::Error> = serde_json::from_str(&req_body);
    match blog_post {
        Ok(blog) => {
            // Check if Blog with given slug already exists or not.
            if db::get_post_by_slug(&conn.lock().unwrap(), &blog.slug)?.is_some() {
                db::delete_post(&conn.lock().unwrap(), &blog.slug)?;
                return Ok(ResponseType::Json(json!({"msg": "Blog deleted successfully", "success" : true})));
            }
            Ok(
                ResponseType::Json(json!({"msg": format!("No blog found with slug - {}", blog.slug), "success" : false}))
            )
        }
        Err(e) => { Ok(ResponseType::Json(json!({"msg" : format!("Failed to Delete blog {}", e), "success" : false}))) }
    }
}
