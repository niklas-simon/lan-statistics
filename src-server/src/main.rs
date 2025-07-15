use actix_web::{App, HttpServer};

mod api;
mod repo;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(api::get_scope())
    })
    .bind(("0.0.0.0", 80))?
    .run()
    .await
}