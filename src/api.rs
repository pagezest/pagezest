use actix_web::{error::{ErrorBadRequest, ErrorInternalServerError, ErrorNotFound,}, web::{self, Data}, HttpResponse, Responder, Result};
use rusqlite::Connection;
use rust_embed::Embed;
use serde_json::{ Value, json };
use std::sync::{ Arc, Mutex } ;

use crate::{db, memory::get_process_memory, plugin_manager::PluginManager, post::BlogPost, render::json_to_html, AppState};

#[derive(Embed)]
#[folder = "admin/dist/"]
struct Asset;

pub async fn update_blog_post(
  app_state: Data<AppState>,
  id: web::Path<String>,
  req_json: web::Json<Value>,
) -> Result<impl Responder> {
  let req_json = req_json.into_inner();
  let conn = app_state.conn.lock().unwrap();
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


  let blog_post: BlogPost = BlogPost::new(slug, title, content);
  // Check if a blog with the given ID exists.
  if db::get_post(&conn, &id, false)
    .map_err(|e| ErrorInternalServerError(e.to_string()))
      ?.is_some() {
        db::update_post(&conn, blog_post, &id, false)
          .map_err(|e| ErrorInternalServerError(e.to_string()))?;
        return Ok(HttpResponse::Ok().json(
            json!({"msg" : "Updated your Blog Successfully", "success" : true})
        ));
  }
  Ok(HttpResponse::Ok().json(
      json!({"msg": format!("No blog found with id - {}", id), "success" : false})
  ))
}


pub async fn find_blog_by_id(
  app_state: Data<AppState>,
  path: web::Path<String>
) -> Result<impl Responder> {
  let conn = app_state.conn.lock()
    .map_err(|e| ErrorInternalServerError(e.to_string()))?;
  let id = path.into_inner();
  let post = db::get_post(&conn, &id, false)
    .or_else(|_| Err(ErrorNotFound("post not found")))?;
    let post: Value = json!({"data": post});
    Ok(HttpResponse::Ok().json(post))
}

pub async fn get_all_blog_posts(
  app_state: Data<AppState>,
) -> Result<impl Responder> {
  let conn = app_state.conn.lock()
    .map_err(|e| ErrorInternalServerError(e.to_string()))?;

  let posts = db::get_all_post(&conn)
    .map_err(|e| ErrorInternalServerError(e.to_string()))?;
    Ok(HttpResponse::Ok().json(json!({"data" : posts, "success": true})))
}

pub async fn create_new_blog_post(
  app_state: Data<AppState>,
  req_json: web::Json<Value>,
) -> Result<impl Responder> {
  let req_json = req_json.into_inner();
  println!("post: {:?}", req_json);
  let conn = app_state.conn.lock()
    .map_err(|e| ErrorInternalServerError(e.to_string()))?;
  let slug = req_json.get("slug")
    .and_then(|v| v.as_str())
    .ok_or_else(|| ErrorInternalServerError(""))?;

  let title = req_json.get("title")
    .and_then(|v| v.as_str())
    .ok_or_else(|| ErrorInternalServerError(""))?;
  let content = req_json
    .get("content")
    .cloned()
    .ok_or_else(|| ErrorInternalServerError(""))?;

  let blog_post: BlogPost = BlogPost::new(slug, title, content);

  // Check if Blog with given slug already exists or not.
  let post = db::get_post(&conn, &blog_post.slug, true)
    .map_err(|e| ErrorInternalServerError(e.to_string()))?;
  if post.is_some() {
    return Err(ErrorBadRequest("slug already exists"));
  }

  db::create_post(&conn, blog_post)
    .map_err(|e| ErrorInternalServerError(e.to_string()))?;
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
    Some(path) => path.into_inner(),
    None => "".to_string(),
  };
  let conn = app_state.conn.lock()
    .map_err(|e| ErrorInternalServerError(e.to_string()))?;
  let plugin_manager = app_state.plugin_manager.clone();
  let post = db::get_post(&conn, &slug, true)
    .map_err(|_e| ErrorNotFound("post not found"))
    .or_else(|_| Err(ErrorNotFound("post not found")))?;

  if post.is_none() {
    return Err(ErrorNotFound("Post not found"));
  }
  let post = post.unwrap();

  let md_json: Value = post.content.clone();
  let md_content = md_json
    .get("json").unwrap();
  let md_content_str = md_content.to_string();
  match render_page(&plugin_manager, &post, &md_content_str, &conn) {
    Ok(resp) => {
      Ok(HttpResponse::Ok().body(resp))
    },
    Err(e) => {
      println!("render error");
      Err(ErrorInternalServerError(e))
    }
  }
}

fn render_page(plugin_manager: &Arc<Mutex<PluginManager>>, post: &BlogPost, content: &str, conn: &Connection) -> Result<String, std::io::Error> {
  let content_json: Value = serde_json::from_str(content)?;

  let mut page_contents = String::new();
  let plugin_manager = plugin_manager.lock().unwrap();
  match json_to_html(post, content, conn, plugin_manager) {
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

