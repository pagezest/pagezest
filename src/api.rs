use actix_web::{error::{ErrorBadRequest, ErrorInternalServerError, ErrorNotFound,}, web::{self, Data}, HttpRequest, HttpResponse, Responder, Result};
use rust_embed::Embed;
use serde_json::{ Value, json };
use std::{str::FromStr, sync::{ Arc, Mutex }} ;

use crate::{memory::get_process_memory, plugin_manager::PluginManager, post::BlogPost, render_flatbuffers::flatbuffers_to_html, AppState};

#[derive(Embed)]
#[folder = "admin/dist/"]
struct Asset;

pub async fn update_blog_post(
  app_state: Data<AppState>,
  id: web::Path<String>,
  req_json: web::Json<Value>,
) -> Result<impl Responder> {
  let req_json = req_json.into_inner();
  let id = id.into_inner();

  let slug = req_json
    .get("slug")
    .and_then(|v| v.as_str())
    .ok_or_else(|| ErrorInternalServerError("slug is missing"))?;

  let title = req_json
    .get("title")
    .and_then(|v| v.as_str())
    .ok_or_else(|| ErrorInternalServerError("title is missing"))?;

  let content = req_json
    .get("content")
    .cloned()
    .ok_or_else(|| ErrorInternalServerError("content is missing"))?;

    let content_md = content
    .get("md")
    .and_then(|v| v.as_str())
    .ok_or_else(|| ErrorInternalServerError("content.md is missing"))?;

    let content_flatbuffer = req_json
        .get("content_flatbuffer64")
    .and_then(|v| v.as_str())
    .ok_or_else(|| ErrorInternalServerError("flatbuffer64 is missing"))?;

    let content_flatbuffer = base64::decode(content_flatbuffer).expect("Failed to decode Base64");


    let blog_post: BlogPost = BlogPost::new(slug, title, content_md, vec![]);
    app_state.posts_fs.update_metadata(&id, &blog_post)
        .map_err(|e| ErrorInternalServerError("error saving metadata"))?;
    app_state.posts_fs.update_content(&id, &content_flatbuffer)
        .map_err(|e| ErrorInternalServerError("error saving content"))?;
  Ok(HttpResponse::Ok().json(
      json!({"msg": format!("No blog found with id - {}", id), "success" : false})
  ))
}


pub async fn find_blog_by_id(
  app_state: Data<AppState>,
  path: web::Path<String>
) -> Result<impl Responder> {
  let id = path.into_inner();
  let (post, _) = app_state.posts_fs.get(&id)
      .map_err(|_e| ErrorNotFound("not found"))?;
    Ok(HttpResponse::Ok().json(post))
}

pub async fn get_all_blog_posts(
  app_state: Data<AppState>,
) -> Result<impl Responder> {

  let posts: Vec<Value> = app_state.posts_fs.list()
      .map_err(|_e| ErrorInternalServerError("Error"))?
      .iter().map(|s| s.to_json())
      .collect();
  Ok(HttpResponse::Ok().json(json!({"data" : posts, "success": true})))
}

pub async fn create_new_blog_post(
  app_state: Data<AppState>,
  req_json: web::Json<Value>,
) -> Result<impl Responder> {
  let req_json = req_json.into_inner();
  println!("post: {:?}", req_json);
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

    let content_md = content
        .get("md")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorInternalServerError(""))?;


    let content_flatbuffer = req_json
      .get("content_flatbuffer64")
      .and_then(|f| f.as_str())
      .ok_or_else(|| ErrorInternalServerError(""))?;

    let content_flatbuffer = base64::decode(content_flatbuffer).expect("Failed to decode Base64");

  let blog_post: BlogPost = BlogPost::new(slug, title, content_md, vec![]);

  // Check if Blog with given slug already exists or not.

  app_state.posts_fs.insert(&blog_post, &content_flatbuffer)
      .map_err(|_e| ErrorInternalServerError("Error"))?;

  Ok(HttpResponse::Ok().json(json!({"msg" : "New Blog Created Successfully", "success": true})))
}

pub async fn delete_blog_post(
  app_state: Data<AppState>,
  path: web::Path<String>
) -> Result<impl Responder> {
  let id = path.into_inner();
  app_state.posts_fs.delete(&id)
      .map_err(|_e| ErrorNotFound("not found"))?;
  Ok(HttpResponse::Ok().json(json!({"msg": "Blog deleted successfully", "success": true})))
}


pub async fn get_server_stats(
  app_state: Data<AppState>,
) -> Result<impl Responder> {
    /*
  let mut stats = db::get_stats(&conn)
    .map_err(|e| ErrorInternalServerError(e.to_string()))?;
    */
    let mut stats = json!({});
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
    None => "_index".to_string(),
  };
  println!("get_post_by_slug {}", slug);
  let plugin_manager = app_state.plugin_manager.clone();
  let post_data = app_state.posts_fs.get_by_slug(&slug)
      .map_err(|_e| ErrorNotFound(format!("Error: {:?}", _e)))?;
    if let Some((mut post, content)) = post_data {
        post.content_flatbuffer = content;


        match render_page(&plugin_manager, &post, &post.content_flatbuffer) {
            Ok(resp) => {
                Ok(HttpResponse::Ok().body(resp))
            },
            Err(e) => {
                println!("render error");
                Err(ErrorInternalServerError(e))
            }
        }
    } else {
        Err(ErrorNotFound(format!("{} Not found", slug)))
    }
}

fn render_page(plugin_manager: &Arc<Mutex<PluginManager>>, post: &BlogPost, content: &Vec<u8>) -> Result<String, std::io::Error> {

  let mut page_contents = String::new();
  let plugin_manager = plugin_manager.lock().unwrap();
  match flatbuffers_to_html(post, content, plugin_manager) {
    Ok(s) => {
      page_contents.push_str(&s);
      return Ok(page_contents);
    },
    Err(e) => {
      println!("run error: {}", e.to_string());
    }

  }
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

