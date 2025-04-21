use rusqlite::Connection;
use serde_json::{Value, json};
use std::{
    fs,
    io::Cursor,
    path::Path,
    sync::{Arc, Mutex},
};
use tiny_http::{Header, Method, Request, Response};

use crate::{db, errors::AppError, mime::get_mime_type, plugin::call_wasm, post::BlogPost};

pub enum ResponseType {
    Json(Value),
    Html(String),
    Binary(Vec<u8>, String), // data, mimeType
}

pub fn route_request(
    method: &Method,
    path: &str,
    conn: &Arc<Mutex<Connection>>,
    request: &mut Request,
) -> Result<ResponseType, AppError> {
    match (method, path) {
        (_, p) if p.starts_with("/pz-admin") => serve_static(request),
        (&Method::Get, p) if p.starts_with("/api/blog/") => find_blog_by_id(conn, p),
        (&Method::Get, "/api/blogs") => get_all_blog_posts(conn),
        (&Method::Post, "/api/blog/new") => create_new_blog_post(conn, request),
        (&Method::Post, "/api/blog/update") => update_blog_post(conn, request),
        (&Method::Post, "/api/plugin/demo") => get_table_of_contents(request),
        (&Method::Delete, p) if p.starts_with("/api/blog/delete/") => delete_blog_post(conn, p),
        (&Method::Get, _) => get_post_by_slug(conn, path),
        _ => Err(AppError::PageNotFound("".to_string())),
    }
}

fn health_check() -> Result<ResponseType, AppError> {
    Ok(ResponseType::Json(
        json!({"msg": "Server is healthy", "success": true}),
    ))
}

fn get_post_by_slug(conn: &Arc<Mutex<Connection>>, p: &str) -> Result<ResponseType, AppError> {
    let slug = p.strip_prefix("/").unwrap_or(p);
    let post = db::get_post(&conn.lock().unwrap(), slug, true)?;
    match post {
        Some(post) => {
            let md_str = post.content; // Assuming `post.content` contains the markdown string
            let md_json: Value = md_str;
            let md_content = md_json.get("md").and_then(|v| v.as_str()).ok_or_else(|| {
                AppError::ServerError("Missing or invalid 'md' field in JSON".to_string())
            })?;
            render_page("plugins/page.json", md_content)
        }
        None => Err(AppError::PageNotFound(format!(
            "No post found for slug: {}",
            slug,
        ))),
    }
}

fn find_blog_by_id(conn: &Arc<Mutex<Connection>>, p: &str) -> Result<ResponseType, AppError> {
    let id = p.strip_prefix("/api/blog/").unwrap();
    let post = db::get_post(&conn.lock().unwrap(), id, false)?;
    match post {
        Some(post) => {
            Ok(ResponseType::Json(
                json!({"data": post})
            ))
        }
        None => Err(AppError::PageNotFound(format!(
            "No post found for id: {}",
            id
        ))),
    }
}


fn render_page(manifest: &str, content: &str) -> Result<ResponseType, AppError> {
    let manifest = std::fs::read(manifest).unwrap();
    let manifest_json: Value = serde_json::from_slice(&manifest)
        .map_err(|e| AppError::ServerError(format!("Failed to parse manifest JSON: {}", e)))?;

    let order = manifest_json
        .get("order")
        .and_then(|o| o.as_array())
        .ok_or_else(|| {
            AppError::ServerError("Missing or invalid 'order' field in manifest".to_string())
        })?
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect::<Vec<String>>();

    let mut page_contents = String::new();
    for val in order {
        if val == "toc" {
            let toc_html = call_wasm("plugins/page.wasm", content, "toc").unwrap();
            page_contents.push_str(&format!("{}\n", toc_html));
        }
    }
    Ok(ResponseType::Html(page_contents))
}

fn get_all_blog_posts(conn: &Arc<Mutex<Connection>>) -> Result<ResponseType, AppError> {
    let posts = db::get_all_post(&conn.lock().unwrap())?;
    Ok(ResponseType::Json(json!({"data" : posts, "success": true})))
}

fn create_new_blog_post(
    conn: &Arc<Mutex<Connection>>,
    request: &mut Request,
) -> Result<ResponseType, AppError> {
    let mut req_body = String::new();
    request.as_reader().read_to_string(&mut req_body).unwrap();

    // Validate request and convert to BlogPost Model.
    let blog_post: Result<BlogPost, serde_json::Error> = serde_json::from_str(&req_body);
    match blog_post {
        Ok(blog) => {
            // Check if Blog with given slug already exists or not.
            if db::get_post(&conn.lock().unwrap(), &blog.slug, true)?.is_some() {
                return Ok(ResponseType::Json(
                    json!({"msg": format!("Blog with slug {} already exists.", blog.slug), "success": false}),
                ));
            }
            db::create_post(&conn.lock().unwrap(), blog)?;
            Ok(ResponseType::Json(
                json!({"msg" : "New Blog Created Successfully", "success": true}),
            ))
        }
        Err(e) => Ok(ResponseType::Json(
            json!({"msg" : format!("Failed to Create blog {}", e), "success" : false}),
        )),
    }
}

fn update_blog_post(
    conn: &Arc<Mutex<Connection>>,
    request: &mut Request,
) -> Result<ResponseType, AppError> {
    let mut req_body = String::new();
    request.as_reader().read_to_string(&mut req_body).unwrap();

    // Parse the request body as JSON.
    let req_json: Value = serde_json::from_str(&req_body)
        .map_err(|e| AppError::ServerError(format!("Failed to parse request body: {}", e)))?;

    // Extract the `id` field from the JSON.
    let id = req_json
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::ServerError("Missing or invalid 'id' field".to_string()))?;

    // Convert the rest of the JSON to a BlogPost model.
    let blog_post: Result<BlogPost, serde_json::Error> = serde_json::from_value(req_json.clone());
    match blog_post {
        Ok(blog) => {
            // Check if a blog with the given ID exists.
            if db::get_post(&conn.lock().unwrap(), id, false)?.is_some() {
                db::update_post(&conn.lock().unwrap(), blog, id, false).unwrap();
                return Ok(ResponseType::Json(
                    json!({"msg" : "Updated your Blog Successfully", "success" : true}),
                ));
            }
            Ok(ResponseType::Json(
                json!({"msg": format!("No blog found with id - {}", id), "success" : false}),
            ))
        }
        Err(e) => Ok(ResponseType::Json(
            json!({"msg" : format!("Failed to Update blog {}", e), "success" : false}),
        )),
    }
}

fn delete_blog_post(conn: &Arc<Mutex<Connection>>, p: &str) -> Result<ResponseType, AppError> {
    let id = p.strip_prefix("/api/blog/delete/").unwrap();

    // Check if a blog with the given ID exists.
    if db::get_post(&conn.lock().unwrap(), id, false)?.is_some() {
        db::delete_post(&conn.lock().unwrap(), id, false)?;
        return Ok(ResponseType::Json(
            json!({"msg": "Blog deleted successfully", "success": true}),
        ));
    }

    Ok(ResponseType::Json(
        json!({"msg": format!("No blog found with id - {}", id), "success": false}),
    ))
}

pub fn load_sample_image() -> Result<Response<Cursor<Vec<u8>>>, AppError> {
    let data = std::fs::read("sample.jpg")?;
    let mut response = Response::from_data(data);
    response.add_header(Header::from_bytes(&b"Content-Type"[..], &b"image/jpeg"[..]).unwrap());
    Ok(response)
}

pub fn get_table_of_contents(request: &mut Request) -> Result<ResponseType, AppError> {
    let mut req_body = String::new();
    request.as_reader().read_to_string(&mut req_body).unwrap();

    let req_json: Value =
        serde_json::from_str(&req_body).map_err(|e| AppError::ServerError(e.to_string()))?;
    let md_content = req_json
        .get("content")
        .and_then(|c| c.as_str())
        .ok_or_else(|| {
            AppError::ServerError("Missing or invalid 'content' field in request body".to_string())
        })?;

    let toc_html = call_wasm("plugins/toc.wasm", md_content, "toc").unwrap();
    Ok(ResponseType::Html(toc_html))
}

fn serve_static(request: &mut Request) -> Result<ResponseType, AppError> {
    let static_serve_path = "pz-admin/";
    let path = request.url().trim_start_matches("/pz-admin").trim_start_matches('/');
    let file_path = Path::new(static_serve_path).join(path);
    let mut file_path = if file_path.is_dir() {
        file_path.join("index.html")
    } else if file_path.is_dir() {
        file_path
    } else {
        file_path
    };
    let file_exists = file_path.exists();
    if !file_exists {
        file_path = Path::new(static_serve_path).join("index.html");
    }
    let file_extension = file_path.extension().and_then(|ext| ext.to_str()).unwrap_or("txt");
    let content = fs::read(&file_path).unwrap();
    Ok(ResponseType::Binary(content, get_mime_type(file_extension).to_string()))
}
