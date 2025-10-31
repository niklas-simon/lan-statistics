use actix_web::{error, put, web, HttpResponse, Responder, Result, Scope};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use chrono::{DateTime};
use common::{response::now_playing::NowPlayingEntry};
use crate::{api::ActixData, config::PASSWORD, repo::now_playing::{get_list, update}};
use serde::Deserialize;

#[derive(Deserialize)]
struct NowPlayingParams {
    last_update: Option<String>
}

#[put("/now-playing")]
async fn put_now_playing(data: ActixData, auth: BearerAuth, body: web::Json<NowPlayingEntry>, query: web::Query<NowPlayingParams>) -> Result<impl Responder> {
    if auth.token() != PASSWORD.as_str() {
        return Err(error::ErrorUnauthorized("unauthorized"));
    }

    let last_update = match query.into_inner().last_update {
        Some(last_update_str) => {
            let Ok(last_update) = DateTime::parse_from_rfc3339(&last_update_str) else {
                return Err(error::ErrorBadRequest("last_update: bad format"));
            };

            Some(last_update)
        },
        None => None
    };

    let now_playing = body.into_inner();
    
    update(&data, now_playing).await;

    if let Some(last_update) = last_update {
        let update_lock = data.last_update.lock().await;

        if *update_lock < last_update {
            return Ok(HttpResponse::NotModified().finish())
        }
    }
    
    Ok(HttpResponse::Ok().json(get_list(data).await))
}

pub fn get_services(scope: Scope) -> Scope {
    scope
        .service(put_now_playing)
}