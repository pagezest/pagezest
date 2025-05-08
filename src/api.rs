use actix_web::{error::{ErrorBadRequest, ErrorInternalServerError, ErrorNotFound,}, web::{self, Data}, HttpRequest, HttpResponse, Responder, Result};
use rusqlite::Connection;
use rust_embed::Embed;
use serde_json::{ Value, json };
use std::{fs, path::Path, sync::{ Arc, Mutex }} ;
use chrono::Utc;

use crate::{db, memory::get_process_memory, plugin_manager::PluginManager, post::BlogPost, render::json_to_html, AppState};

#[derive(Embed)]
#[folder = "admin/dist/"]
struct Asset;

pub async fn update_blog_post(
  _app_state: Data<AppState>,
  id: web::Path<String>,
  req_json: web::Json<Value>,
) -> Result<impl Responder> {
  let req_json = req_json.into_inner();
  let id = id.into_inner();

  let slug = req_json
    .get("slug")
    .and_then(|v| v.as_str())
    .ok_or_else(|| ErrorInternalServerError(""))?;

  let title = req_json
    .get("title")
    .and_then(|v| v.as_str())
    .ok_or_else(|| ErrorInternalServerError(""))?;

  let content = req_json
    .get("content")
    .cloned()
    .ok_or_else(|| ErrorInternalServerError(""))?;


  let blog_dir = Path::new("posts").join(slug);
  if !blog_dir.exists() {
      return Err(ErrorNotFound("Post not found"));
  }

  let metadata_file = blog_dir.join("metadata.json");
  let content_file = blog_dir.join("content.json");
  let metadata = json!({
      "slug": slug,
      "title": title,
      "created_at": Utc::now().to_string(),
      "updated_at": Utc::now().to_string(),
  });
  fs::write(metadata_file, metadata.to_string())?;
  fs::write(content_file, content.to_string())?;

  Ok(HttpResponse::Ok().json(
      json!({"msg": format!("No blog found with id - {}", id), "success" : false})
  ))
}


pub async fn find_blog_by_id(
  app_state: Data<AppState>,
  path: web::Path<String>
) -> Result<impl Responder> {
  let id = path.into_inner();
  let blog_dir = Path::new("posts").join(id);
  let metadata_file = blog_dir.join("metadata.json");
  let content_file = blog_dir.join("content.json");
  if !metadata_file.exists() || !content_file.exists() {
      return Err(ErrorNotFound("Post not found"));
  }

  let metadata = std::fs::read_to_string(metadata_file)?;
  let mut post: Value = serde_json::from_str(&metadata)?;

  let content = std::fs::read_to_string(content_file)?;
  let content: Value = serde_json::from_str(&content)?;
  post["content"] = content;

  let post: Value = json!({"data": post});
  Ok(HttpResponse::Ok().json(post))
}

pub async fn get_all_blog_posts(
  _app_state: Data<AppState>,
) -> Result<impl Responder> {
    let blogs_dir = Path::new("posts");
    let mut posts: Vec<Value> = Vec::new();
    if let Ok(entries) = fs::read_dir(blogs_dir) {
        for entry in entries.filter_map(Result::ok).map(|a| a.path()).filter(|a| a.is_dir()) {
            let slug = entry.file_name().unwrap();
            let metadata_file = entry.join("metadata.json");
            let content_file = entry.join("content.json");
            if !metadata_file.exists() || !content_file.exists() { continue; }
            let metadata = fs::read_to_string(metadata_file)?;
            let mut metadata: Value = serde_json::from_str(&metadata)?;
            metadata["slug"] = Value::from(slug.to_str());
            metadata["id"] = Value::from(slug.to_str());
            posts.push(metadata);
        }
    }

    Ok(HttpResponse::Ok().json(json!({"data": posts})))
}

pub async fn create_new_blog_post(
  _app_state: Data<AppState>,
  req_json: web::Json<Value>,
) -> Result<impl Responder> {
  let req_json = req_json.into_inner();
  let slug = req_json.get("slug")
    .and_then(|v| v.as_str())
    .ok_or_else(|| ErrorInternalServerError(""))?;

  let title = req_json.get("title")
    .and_then(|v| v.as_str())
    .ok_or_else(|| ErrorInternalServerError(""))?;
  let content = req_json
    .get("content")
    .ok_or_else(|| ErrorInternalServerError(""))?;


  let blog_dir = Path::new("posts").join(slug);
  if blog_dir.exists() {
      return Err(ErrorBadRequest("slug already exists"));
  }
  fs::create_dir(&blog_dir)?;

  let metadata_file = blog_dir.join("metadata.json");
  let content_file = blog_dir.join("content.json");
  let metadata = json!({
      "slug": slug,
      "title": title,
      "created_at": Utc::now().to_string(),
      "updated_at": Utc::now().to_string(),
  });
  fs::write(metadata_file, metadata.to_string())?;
  fs::write(content_file, content.to_string())?;

  Ok(HttpResponse::Ok().json(json!({"msg" : "New Blog Created Successfully", "success": true})))
}

pub async fn delete_blog_post(
  app_state: Data<AppState>,
  path: web::Path<String>
) -> Result<impl Responder> {
  let id = path.into_inner();
  let conn = app_state.conn.lock()
    .map_err(|e| ErrorInternalServerError(e.to_string()))?;

  // Check if a blog with the given ID exists.
  _ = db::get_post(&conn, &id, false)
    .map_err(|e| ErrorInternalServerError(e.to_string()))?;
    db::delete_post(&conn, &id, false)
      .map_err(|e| ErrorInternalServerError(e.to_string()))?;
    Ok(HttpResponse::Ok().json(json!({"msg": "Blog deleted successfully", "success": true})))
}


pub async fn get_server_stats(
  app_state: Data<AppState>,
) -> Result<impl Responder> {
  let conn = app_state.conn.lock()
    .map_err(|e| ErrorInternalServerError(e.to_string()))?;
  let mut stats = db::get_stats(&conn)
    .map_err(|e| ErrorInternalServerError(e.to_string()))?;
  let memory = get_process_memory();
  stats["memory"] = json!(memory);
  Ok(HttpResponse::Ok().json(json!({"data": stats})))
}

pub async fn get_post_by_slug(
  app_state: Data<AppState>,
  path: Option<web::Path<String>>
) -> Result<impl Responder> {
  let slug = match path {
    Some(path) => &path.into_inner(),
    None => "",
  };
  let conn = app_state.conn.lock()
    .map_err(|e| ErrorInternalServerError(e.to_string()))?;
  let plugin_manager = app_state.plugin_manager.clone();



  let blog_file = Path::new("posts").join(slug);
  let metadata_file = blog_file.join("metadata.json");
  let content_file = blog_file.join("content.json");
  if !metadata_file.exists() || !blog_file.exists() || !content_file.exists() {
      return Err(ErrorNotFound("Post not found"));
  }

  let metadata = std::fs::read_to_string(metadata_file)?;
  let metadata: Value = serde_json::from_str(&metadata)?;

  let content = std::fs::read_to_string(content_file)?;
  let content: Value = serde_json::from_str(&content)?;
  let content = content.get("json").unwrap();

  let post = BlogPost::new(slug,
      metadata.get("title").and_then(|s| s.as_str()).unwrap(),
      content.clone(),
  );


  match render_page(&plugin_manager, &post, &content, &conn) {
    Ok(resp) => {
      Ok(HttpResponse::Ok().body(resp))
    },
    Err(e) => {
      println!("render error");
      Err(ErrorInternalServerError(e))
    }
  }
}

fn render_page(plugin_manager: &Arc<Mutex<PluginManager>>, post: &BlogPost, content_json: &Value, conn: &Connection) -> Result<String, std::io::Error> {

  let mut page_contents = String::new();
  let plugin_manager = plugin_manager.lock().unwrap();
  let content = content_json.to_string();
  match json_to_html(post, &content, conn, plugin_manager) {
    Ok(s) => {
      page_contents.push_str(&s);
      return Ok(page_contents);
    },
    Err(e) => {
      println!("run error: {}", e.to_string());
    }

  }
  page_contents.push_str("<pre>");
  page_contents.push_str(&serde_json::to_string_pretty(&content_json)?);
  page_contents.push_str("</pre>");
  Ok(page_contents)
}

pub async fn serve_embedded(req: HttpRequest) -> impl Responder {
  let path = req.match_info().query("tail").trim_start_matches('/');
  let path = if path.is_empty() { "index.html" } else { path };

  match Asset::get(path) {
    Some(content) => {
      let body = actix_web::body::BoxBody::new(content.data.to_vec());
      let mime = mime_guess::from_path(path).first_or_octet_stream();
      HttpResponse::Ok().content_type(mime.as_ref()).body(body)
    }
    None => HttpResponse::NotFound().body("Not Found"),
  }
}

