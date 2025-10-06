use actix_web::{error, put, web, HttpResponse, Responder, Result, Scope};
use chrono::{Local, NaiveDateTime};
use common::{response::now_playing::NowPlayingEntry};
use crate::repo::now_playing::{get_list, update, LAST_UPDATE};
use serde::Deserialize;

#[derive(Deserialize)]
struct NowPlayingParams {
    last_update: Option<String>
}

#[put("/now-playing")]
async fn put_now_playing(body: web::Json<NowPlayingEntry>, query: web::Query<NowPlayingParams>) -> Result<impl Responder> {
    let now_playing = body.into_inner();
    
    update(now_playing).await;

    if let Some(last_update_str) = query.into_inner().last_update {
        let Ok(last_update) = NaiveDateTime::parse_from_str(&last_update_str, "%Y-%m-%dT%H:%M:%S") else {
            return Err(error::ErrorBadRequest("last_update must be in format YYYY-MM-dd'T'HH:mm:ss"));
        };

        let update_lock = LAST_UPDATE.lock().await;

        if *update_lock < last_update.and_local_timezone(Local).earliest().unwrap() {
            return Ok(HttpResponse::NotModified().finish())
        }
    }
    
    Ok(HttpResponse::Ok().json(get_list().await))
}

pub fn get_services(scope: Scope) -> Scope {
    scope
        .service(put_now_playing)
}