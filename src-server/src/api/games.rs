use std::{fs::File, hash::{DefaultHasher, Hash, Hasher}, io::Read};

use actix_web::{error::ErrorNotFound, get, web, HttpResponse, Responder, Result, Scope};
use common::{game::Game, response::games::GamesResponse};
use serde::{Deserialize};

#[derive(Deserialize)]
struct GamesParams {
    hash: Option<u64>
}

#[get("/games")]
async fn games(query: web::Query<GamesParams>) -> Result<impl Responder> {
    let mut buf = String::new();

    File::open("games.json")
        .map_err(ErrorNotFound)?
        .read_to_string(&mut buf)
        .map_err(ErrorNotFound)?;

    let mut hasher = DefaultHasher::new();

    buf.hash(&mut hasher);

    let hash = hasher.finish();

    if query.hash == Some(hash) {
        return Ok(HttpResponse::NotModified().finish());
    }

    let json = serde_json::from_str::<Vec<Game>>(&buf)
        .map_err(ErrorNotFound)?;
    
    Ok(HttpResponse::Ok().json(GamesResponse {
        games: json,
        hash
    }))
}

pub fn get_services(scope: Scope) -> Scope {
    scope
        .service(games)
}