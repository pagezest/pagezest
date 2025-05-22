use actix_web::{
    error::{ErrorBadRequest, ErrorInternalServerError, ErrorNotFound},
    web::{self, Data},
    HttpRequest, HttpResponse, Responder, Result,
};
use rust_embed::Embed;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::{Arc, RwLock};

use crate::{
    db::{self, DBPool},
    inmemory_cache::CacheSet,
    plugin_manager::PluginManager,
    post::BlogPost,
    render_flatbuffers::{flatbuffers_prerender, flatbuffers_to_html},
    AppState,
};

#[derive(Embed)]
#[folder = "admin/dist/"]
struct Asset;

#[derive(Debug, Serialize, Deserialize)]
pub struct BlogPostInput {
    slug: String,
    title: String,
    content: String,
    content_flatbuffer64: String,
}

pub async fn update_blog_post(
    app_state: Data<AppState>,
    id: web::Path<String>,
    req_json: web::Json<BlogPostInput>,
) -> Result<impl Responder> {
    let req_json = req_json.into_inner();
    let id = id.into_inner();

    if req_json.content_flatbuffer64.len() == 0 {
        return Err(ErrorBadRequest("content_flatbuffer64 is missing"));
    }

    #[allow(deprecated)]
    let content_flatbuffer =
        base64::decode(req_json.content_flatbuffer64).expect("Failed to decode Base64");

    app_state.cache.do_send(CacheSet::Remove {
        key: req_json.slug.clone(),
    });

    let mut blog_post: BlogPost = BlogPost::new(
        &req_json.slug,
        &req_json.title,
        &req_json.content,
        content_flatbuffer.clone(),
    );
    let content_cached = flatbuffers_prerender(&blog_post, &content_flatbuffer)?;
    blog_post.content_cached = content_cached;

    // Check if a blog with the given ID exists.
    if db::get_post_by_id(&app_state.conn, &id)
        .await
        .map_err(|e| ErrorInternalServerError(e.to_string()))?
        .is_some()
    {
        db::update_post(&app_state.conn, blog_post, &id)
            .await
            .map_err(|e| ErrorInternalServerError(e.to_string()))?;
        return Ok(HttpResponse::Ok()
            .json(json!({"msg" : "Updated your Blog Successfully", "success" : true})));
    }
    Ok(HttpResponse::Ok()
        .json(json!({"msg": format!("No blog found with id - {}", id), "success" : false})))
}

pub async fn find_blog_by_id(
    app_state: Data<AppState>,
    path: web::Path<String>,
) -> Result<impl Responder> {
    let id = path.into_inner();
    println!("find_blog_by_id: {}", id);
    let post = db::get_post_by_id(&app_state.conn, &id)
        .await
        .or_else(|e| Err(ErrorNotFound(e.to_string())))?;
    let post: serde_json::Value = json!({ "data": post });
    Ok(HttpResponse::Ok().json(post))
}

pub async fn get_all_blog_posts(app_state: Data<AppState>) -> Result<impl Responder> {
    let posts = db::get_all_post(&app_state.conn)
        .await
        .map_err(|e| ErrorInternalServerError(e.to_string()))?;
    Ok(HttpResponse::Ok().json(json!({"data" : posts, "success": true})))
}

pub async fn create_new_blog_post(
    app_state: Data<AppState>,
    req_json: web::Json<BlogPostInput>,
) -> Result<impl Responder> {
    let req_json = req_json.into_inner();
    /*
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
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorInternalServerError(""))?;

    let content_flatbuffer = req_json
        .get("content_flatbuffer64")
        .and_then(|f| f.as_str())
        .ok_or_else(|| ErrorInternalServerError(""))?;
    */

    #[allow(deprecated)]
    let content_flatbuffer =
        base64::decode(req_json.content_flatbuffer64).expect("Failed to decode Base64");

    let mut blog_post: BlogPost = BlogPost::new(
        &req_json.slug,
        &req_json.title,
        &req_json.content,
        content_flatbuffer.clone(),
    );
    let content_cached = flatbuffers_prerender(&blog_post, &content_flatbuffer)?;

    // Check if Blog with given slug already exists or not.
    let post = db::get_post_by_slug(&app_state.conn, &blog_post.slug)
        .await
        .map_err(|e| ErrorInternalServerError(e.to_string()))?;
    if post.is_some() {
        return Err(ErrorBadRequest("slug already exists"));
    }

    blog_post.content_cached = content_cached;
    db::create_post(&app_state.conn, blog_post)
        .await
        .map_err(|e| ErrorInternalServerError(e.to_string()))?;
    Ok(HttpResponse::Ok().json(json!({"msg" : "New Blog Created Successfully", "success": true})))
}

pub async fn delete_blog_post(
    app_state: Data<AppState>,
    path: web::Path<String>,
) -> Result<impl Responder> {
    let id = path.into_inner();

    // Check if a blog with the given ID exists.
    let post = db::get_post_by_id(&app_state.conn, &id)
        .await
        .map_err(|e| ErrorInternalServerError(e.to_string()))?;

    if post.is_some() {
        app_state.cache.do_send(CacheSet::Remove {
            key: post.unwrap().slug,
        });
    }
    db::delete_post(&app_state.conn, &id, false)
        .await
        .map_err(|e| ErrorInternalServerError(e.to_string()))?;
    Ok(HttpResponse::Ok().json(json!({"msg": "Blog deleted successfully", "success": true})))
}

/*
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
*/

pub async fn get_post_by_slug(
    app_state: Data<AppState>,
    path: Option<web::Path<String>>,
) -> Result<impl Responder> {
    let slug = match path {
        Some(path) => path.into_inner(),
        None => "".to_string(),
    };

    let conn = app_state.conn.clone();
    let plugin_manager = app_state.plugin_manager.clone();

    let post = match app_state
        .cache
        .send(CacheSet::Get { key: slug.clone() })
        .await
    {
        Ok(Some(post)) => post,
        _ => {
            let post = db::get_post_by_slug(&conn, &slug)
                .await
                .map_err(|_| ErrorNotFound("Post not found"))?
                .ok_or_else(|| ErrorNotFound("Post not found"))?;
            app_state.cache.do_send(CacheSet::Insert {
                key: slug.clone(),
                value: post.clone(),
            });
            post
        }
    };

    match render_page(&plugin_manager, &post, &post.content_cached, &conn).await {
        Ok(resp) => Ok(HttpResponse::Ok().body(resp)),
        Err(e) => {
            println!("render error");
            Err(ErrorInternalServerError(e))
        }
    }
}

async fn render_page(
    plugin_manager: &Arc<RwLock<PluginManager>>,
    post: &BlogPost,
    content: &Vec<u8>,
    conn: &DBPool,
) -> Result<String, std::io::Error> {
    let mut page_contents = String::new();
    match flatbuffers_to_html(post, content, conn, plugin_manager).await {
        Ok(s) => {
            page_contents.push_str(&s);
            return Ok(page_contents);
        }
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
