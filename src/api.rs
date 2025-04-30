use rusqlite::Connection;
use rust_embed::Embed;
use serde_json::{ Value, json };
use std::{ error::Error, fs, io::Cursor, path::Path, sync::{ Arc, Mutex } };
use tiny_http::{ Header, Method, Request, Response };

use crate::{ db, errors::AppError, mime::get_mime_type, plugin::call_wasm, plugin_manager::{Plugin, PluginManager}, post::BlogPost, render::json_to_html };

pub enum ResponseType {
    Json(Value),
    Html(String),
    Binary(Vec<u8>, String), // data, mimeType
}

#[derive(Embed)]
#[folder = "admin/dist/"]
struct Asset;

pub fn route_request(
    method: &Method,
    path: &str,
    conn: &Arc<Mutex<Connection>>,
    plugin_manager: &Arc<Mutex<PluginManager>>,
    request: &mut Request
) -> Result<ResponseType, AppError> {
    match (method, path) {
        (_, p) if p.starts_with("/pz-admin") => serve_static(request),
        (&Method::Get, p) if p.starts_with("/api/blog/") => find_blog_by_id(conn, p),
        (&Method::Get, "/api/blogs") => get_all_blog_posts(conn),
        (&Method::Post, "/api/blog/new") => create_new_blog_post(conn, request),
        (&Method::Post, "/api/blog/update") => update_blog_post(conn, request),
        (&Method::Post, "/api/plugin/demo") => get_table_of_contents(request),
        (&Method::Delete, p) if p.starts_with("/api/blog/delete/") => delete_blog_post(conn, p),
        (_, _) if path.starts_with("/api") => not_implemented_error(request),
        (&Method::Get, _) => get_post_by_slug(conn, plugin_manager, path),
        _ => Err(AppError::PageNotFound("".to_string())),
    }
}

fn get_post_by_slug(conn: &Arc<Mutex<Connection>>, plugin_manager: &Arc<Mutex<PluginManager>>, p: &str) -> Result<ResponseType, AppError> {
    let slug = p.strip_prefix("/").unwrap_or(p);
    let post = db::get_post(&conn.lock().unwrap(), slug, true)?;
    match post {
        Some(post) => {
            let md_json: Value = post.content;
            let md_content = md_json
                .get("json")
                .ok_or_else(|| {
                    AppError::ServerError(
                        "Missing or invalid 'json' field in blog contents".to_string()
                    )
                })?;
            let md_content_str = md_content.to_string();
            render_page(plugin_manager, &md_content_str)
        }
        None => Err(AppError::PageNotFound(format!("No post found for slug: {}", slug))),
    }
}

fn find_blog_by_id(conn: &Arc<Mutex<Connection>>, p: &str) -> Result<ResponseType, AppError> {
    let id = p.strip_prefix("/api/blog/").unwrap();
    let post = db::get_post(&conn.lock().unwrap(), id, false)?;
    match post {
        Some(post) => Ok(ResponseType::Json(json!({"data": post}))),
        None => Err(AppError::PageNotFound(format!("No post found for id: {}", id))),
    }
}

fn render_page(plugin_manager: &Arc<Mutex<PluginManager>>, content: &str) -> Result<ResponseType, AppError> {
    let content_json: Value = serde_json::from_str(content).unwrap();

    let mut page_contents = String::new();
    let plugin_manager = plugin_manager.lock().unwrap();
    match json_to_html(content, plugin_manager) {
        Ok(s) => {
            page_contents.push_str(&s);
            return Ok(ResponseType::Html(page_contents));
            //page_contents.push_str(&json_to_html(content, plugin_manager).unwrap());
        },
        Err(e) => {
            println!("run error: {}", e.to_string());
        }
        
    }
    page_contents.push_str("<pre>");
    page_contents.push_str(&serde_json::to_string_pretty(&content_json).unwrap());
    page_contents.push_str("</pre>");
    Ok(ResponseType::Html(page_contents))
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
    let req_json: Value = serde_json
        ::from_str(&req_body)
        .map_err(|_| AppError::PageNotFound("Invalid request body".to_string()))?;

    let slug = req_json
        .get("slug")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::PageNotFound("Missing 'slug' field".to_string()))?;

    let title = req_json
        .get("title")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::PageNotFound("Missing 'title' field".to_string()))?;

    let content = req_json
        .get("content")
        .cloned()
        .ok_or_else(|| AppError::PageNotFound("Missing 'content' field".to_string()))?;

    let blog_post: BlogPost = BlogPost::new(slug, title, content);

    // Check if Blog with given slug already exists or not.
    if db::get_post(&conn.lock().unwrap(), &blog_post.slug, true)?.is_some() {
        return Ok(
            ResponseType::Json(
                json!({"msg": format!("Blog with slug {} already exists.", blog_post.slug), "success": false})
            )
        );
    }
    db::create_post(&conn.lock().unwrap(), blog_post)?;
    Ok(ResponseType::Json(json!({"msg" : "New Blog Created Successfully", "success": true})))
}

fn update_blog_post(
    conn: &Arc<Mutex<Connection>>,
    request: &mut Request
) -> Result<ResponseType, AppError> {
    let mut req_body = String::new();
    request.as_reader().read_to_string(&mut req_body).unwrap();

    // Parse the request body as JSON.
    let req_json: Value = serde_json
        ::from_str(&req_body)
        .map_err(|e| AppError::ServerError(format!("Failed to parse request body: {}", e)))?;

    // Extract the `id` field from the JSON.
    let id = req_json
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::ServerError("Missing or invalid 'id' field".to_string()))?;

    // Validate request and convert to BlogPost Model.
    let req_json: Value = serde_json
        ::from_str(&req_body)
        .map_err(|_| AppError::PageNotFound("Invalid request body".to_string()))?;

    let slug = req_json
        .get("slug")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::PageNotFound("Missing 'slug' field".to_string()))?;

    let title = req_json
        .get("title")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::PageNotFound("Missing 'title' field".to_string()))?;

    let content = req_json
        .get("content")
        .cloned()
        .ok_or_else(|| AppError::PageNotFound("Missing 'content' field".to_string()))?;

    let blog_post: BlogPost = BlogPost::new(slug, title, content);
    // Check if a blog with the given ID exists.
    if db::get_post(&conn.lock().unwrap(), id, false)?.is_some() {
        db::update_post(&conn.lock().unwrap(), blog_post, id, false).unwrap();
        return Ok(
            ResponseType::Json(json!({"msg" : "Updated your Blog Successfully", "success" : true}))
        );
    }
    Ok(
        ResponseType::Json(
            json!({"msg": format!("No blog found with id - {}", id), "success" : false})
        )
    )
}

fn delete_blog_post(conn: &Arc<Mutex<Connection>>, p: &str) -> Result<ResponseType, AppError> {
    let id = p.strip_prefix("/api/blog/delete/").unwrap();

    // Check if a blog with the given ID exists.
    if db::get_post(&conn.lock().unwrap(), id, false)?.is_some() {
        db::delete_post(&conn.lock().unwrap(), id, false)?;
        return Ok(ResponseType::Json(json!({"msg": "Blog deleted successfully", "success": true})));
    }

    Ok(
        ResponseType::Json(
            json!({"msg": format!("No blog found with id - {}", id), "success": false})
        )
    )
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

    let req_json: Value = serde_json
        ::from_str(&req_body)
        .map_err(|e| AppError::ServerError(e.to_string()))?;
    let md_content = req_json
        .get("content")
        .and_then(|c| c.as_str())
        .ok_or_else(|| {
            AppError::ServerError("Missing or invalid 'content' field in request body".to_string())
        })?;

    let toc_html = call_wasm("plugins/toc.wasm", md_content, "toc").unwrap();
    Ok(ResponseType::Html(toc_html))
}

fn serve_static_fs(request: &mut Request) -> Result<ResponseType, AppError> {
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
    let file_extension = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("txt");
    if file_path.exists() {
        let content = fs::read(&file_path).unwrap();
        return Ok(ResponseType::Binary(content, get_mime_type(file_extension).to_string()))
    }
    Err(AppError::PageNotFound("not found".to_string()))
}

fn serve_static(request: &mut Request) -> Result<ResponseType, AppError> {
    let path = request.url().trim_start_matches("/pz-admin").trim_start_matches('/');
    let mut file_path = Path::new(path);

    let file_exists = match Asset::get(file_path.to_str().unwrap()) {
        Some(_) => true,
        None => false
    };

    if !file_exists {
        file_path = Path::new("index.html");
    }
    let file_extension = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("txt");
    if let Some(content) = Asset::get(file_path.to_str().unwrap()) {
        let content = std::str::from_utf8(&content.data.clone()).unwrap().to_string();
        return Ok(ResponseType::Binary(content.as_bytes().to_vec(), get_mime_type(file_extension).to_string()))
    }
    Err(AppError::PageNotFound("not found".to_string()))
}

fn not_implemented_error(request: &mut Request) -> Result<ResponseType, AppError> {
    Err(
        AppError::ServerError("Not implemented yet".to_string())
    )
}
