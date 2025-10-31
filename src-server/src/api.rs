use std::{collections::HashMap, sync::Arc};

use actix_web::{Scope, web::{self, Data}};
use chrono::{DateTime, Local};
use tokio::sync::Mutex;

use crate::{metrics::MetricsContext, repo::now_playing::NowPlayingInfo};

mod now_playing;
mod games;

#[derive(Clone)]
pub struct SharedData {
    pub store: Arc<Mutex<HashMap<String, NowPlayingInfo>>>,
    pub last_update: Arc<Mutex<DateTime<Local>>>,
    pub metrics: MetricsContext
}

pub type ActixData = Data<SharedData>;

impl SharedData {
    pub fn new() -> SharedData {
        SharedData {
            store: Arc::new(Mutex::new(HashMap::new())),
            last_update: Arc::new(Mutex::new(Local::now())),
            metrics: MetricsContext::new()
        }
    }
}

pub fn get_scope() -> Scope {
    let mut scope = web::scope("/api/v1");

    scope = now_playing::get_services(scope);
    scope = games::get_services(scope);

    scope
}