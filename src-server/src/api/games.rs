use actix_files::NamedFile;
use actix_web::{error::ErrorNotFound, get, web::Path, HttpResponse, Responder, Result, Scope};
use common::{response::games::GamesResponse};
use serde::Deserialize;

use crate::{config::ICONS_DIR, repo::games::{get_game, get_games}};

#[derive(Deserialize)]
struct GameIconParameters {
    name: String
}

#[get("/games")]
async fn games() -> Result<impl Responder> {    
    Ok(HttpResponse::Ok().json(GamesResponse {
        games: get_games().to_vec()
    }))
}

#[get("/games/{name}/icon")]
async fn game_icon(path: Path<GameIconParameters>) -> Result<impl Responder> {
    let Some(game) = get_game(&path.name) else {
        return Err(ErrorNotFound("not found"));
    };

    Ok(NamedFile::open(format!("{}/{}", ICONS_DIR.as_str(), game.icon)))
}

pub fn get_services(scope: Scope) -> Scope {
    scope
        .service(games)
        .service(game_icon)
}