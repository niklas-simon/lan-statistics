use std::sync::LazyLock;

use log::{error, info, warn};
use spacetimedb_sdk::{credentials, DbContext, Error, Identity};
use tokio::sync::Mutex;

use crate::{app, config::get_config_or_default, module_bindings::{client_connected, set_games, set_userinfo, DbConnection, ErrorContext, SubscriptionEventContext}};

pub static CTX: LazyLock<Mutex<Option<DbConnection>>> = LazyLock::new(|| Mutex::new(None));

pub async fn connect() -> Result<(), String> {
    match connect_to_db().await {
        Ok(conn) => {
            register_callbacks(&conn);
            subscribe_to_tables(&conn);
            conn.run_threaded();
            let mut ctx_lock = CTX.lock().await;
            *ctx_lock = Some(conn);
            Ok(())
        },
        Err(err) => Err(err)
    }
}

pub fn reset_ctx() {
    tokio::task::spawn(async {
        let mut ctx_lock = CTX.lock().await;
        *ctx_lock = None
    });
}

/// Load credentials from a file and connect to the database.
async fn connect_to_db() -> Result<DbConnection, String> {
    let creds = creds_store().load()
        .map_err(|e| e.to_string())?;
    println!("creds: {:?}", creds);
    let config = get_config_or_default();
    DbConnection::builder()
        .on_connect(on_connected)
        .on_connect_error(on_connect_error)
        .on_disconnect(on_disconnected)
        .with_token(creds)
        .with_module_name(config.remote_db)
        .with_uri(config.remote_url)
        .build()
        .map_err(|e| e.to_string())
}

pub fn creds_store() -> credentials::File {
    credentials::File::new("lan-tracker")
}

fn on_connected(_ctx: &DbConnection, _identity: Identity, token: &str) {
    if let Err(e) = creds_store().save(token) {
        error!("Failed to save credentials: {:?}", e);
    }
}

fn on_connect_error(_ctx: &ErrorContext, err: Error) {
    error!("Connection error: {:?}", err);

    reset_ctx();
}

fn on_disconnected(_ctx: &ErrorContext, err: Option<Error>) {
    if let Some(err) = err {
        error!("Disconnected: {}", err);
    } else {
        warn!("Disconnected.");
    }

    reset_ctx();
}

fn register_callbacks(ctx: &DbConnection) {
    ctx.reducers.on_set_games(|ctx, _| app::send_games_update(&ctx.db));
    ctx.reducers.on_client_connected(|ctx| app::send_games_update(&ctx.db));
    ctx.reducers.on_set_userinfo(|ctx, _| app::send_games_update(&ctx.db));
}

fn subscribe_to_tables(ctx: &DbConnection) {
    ctx.subscription_builder()
        .on_applied(on_sub_applied)
        .on_error(on_sub_error)
        .subscribe(["SELECT * FROM user", "SELECT * FROM game"]);
}

fn on_sub_applied(ctx: &SubscriptionEventContext) {
    app::send_games_update(&ctx.db);
    info!("Fully connected and all subscriptions applied.");
}

fn on_sub_error(_ctx: &ErrorContext, err: Error) {
    warn!("Subscription failed: {}", err);
}