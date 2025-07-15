use actix_web::{web, Scope};

mod now_playing;

pub fn get_scope() -> Scope {
    let mut scope = web::scope("/api/v1");

    scope = now_playing::get_services(scope);

    scope
}