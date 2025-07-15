use actix_web::{error, get, put, web, HttpResponse, Responder, Result, Scope};
use chrono::{Local, NaiveDateTime};
use common::game::Game;
use crate::repo::now_playing::{get_list, update, LAST_UPDATE};
use serde::Deserialize;

#[derive(Deserialize)]
struct NowPlayingParams {
    last_update: Option<String>
}

#[get("/now-playing")]
async fn get_now_playing(query: web::Query<NowPlayingParams>) -> Result<impl Responder> {
    if let Some(last_update_str) = query.into_inner().last_update {
        let Ok(last_update) = NaiveDateTime::parse_from_str(&last_update_str, "%Y-%m-%dT%H:%M:%S") else {
            return Err(error::ErrorBadRequest("last_update must be in format YYYY-MM-dd'T'HH:mm:ss"));
        };

        let update_lock = LAST_UPDATE.lock()
            .map_err(|e| error::ErrorInternalServerError(String::from("now_playing: error getting update lock: ") + &e.to_string()))?;

        if *update_lock < last_update.and_local_timezone(Local).earliest().unwrap() {
            return Ok(HttpResponse::NotModified().body(""))
        }
    }
    
    get_list()
        .map(|res| HttpResponse::Ok().json(res))
        .map_err(|err| error::ErrorInternalServerError(err))
}

#[put("/now-playing/{player}")]
async fn put_now_playing(player: web::Path<String>, games: web::Json<Vec<Game>>) -> Result<()> {
    update(player.into_inner(), games.into_inner())
        .map_err(|err| error::ErrorInternalServerError(err))
}

pub fn get_services(scope: Scope) -> Scope {
    scope
        .service(get_now_playing)
        .service(put_now_playing)
}