use actix_files::Files;
use actix_web::web;

use crate::api;

pub fn config(cfg: &mut web::ServiceConfig) {
  cfg
    .route("/api/blog{trailing:/?}", web::post().to(api::create_new_blog_post))
    .route("/api/blog/{id}", web::get().to(api::find_blog_by_id))
    .route("/api/blog/{id}", web::put().to(api::update_blog_post))
    .route("/api/blog/{id}", web::delete().to(api::delete_blog_post))
    .route("/api/blog{trailing:/?}", web::get().to(api::get_all_blog_posts))
    .route("/api/stats", web::get().to(api::get_server_stats))
    .route("/{tail:.*}", web::get().to(api::get_post_by_slug))
    .service(Files::new("/pz-admin", "./pz-admin").show_files_listing().redirect_to_slash_directory());
}
