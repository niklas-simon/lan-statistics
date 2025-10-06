use log::info;
use spacetimedb::{reducer, table, Identity, ReducerContext, SpacetimeType, Table};

#[derive(SpacetimeType, Debug)]
pub struct UserInfo {
    name: String,
    email: String,
    ip: String,
    picture: String
}

#[table(name = user, public)]
pub struct User {
    #[primary_key]
    identity: Identity,
    name: String,
    email: String,
    ip: String,
    picture: String
}

#[table(name = game, public)]
pub struct Game {
    user: Identity,
    name: String,
}

#[reducer]
pub fn set_userinfo(ctx: &ReducerContext, info: UserInfo) -> Result<(), String> {
    info!("set_userinfo called for {:?}", info);

    let user = User {
        name: info.name,
        email: info.email,
        ip: info.ip,
        picture: info.picture,
        identity: ctx.sender
    };

    if ctx.db.user().identity().find(ctx.sender).is_some() {
        ctx.db.user().identity().update(user);
    } else {
        ctx.db.user().insert(user);
    }

    Ok(())
}

#[reducer]
pub fn set_games(ctx: &ReducerContext, names: Vec<String>) -> Result<(), String> {
    ctx.db.game().iter()
        .filter(|g| g.user == ctx.sender)
        .for_each(|g| {
            ctx.db.game().delete(g);
        });
    
    for name in names.iter() {
        ctx.db.game().insert(Game {
            user: ctx.sender,
            name: name.clone()
        });
    }

    Ok(())
}

#[reducer(client_connected)]
pub fn client_connected(ctx: &ReducerContext) {
    let user = ctx.db.user().identity().find(ctx.sender);
    if let Some(user) = &user {
        log::info!("user {} connected", user.name);
    } else {
        log::info!("unknown user {} connected", String::from(ctx.sender.to_abbreviated_hex()));
    }
}

#[reducer(client_disconnected)]
pub fn identity_disconnected(ctx: &ReducerContext) {
    let user = ctx.db.user().identity().find(ctx.sender);
    if let Some(user) = &user {
        ctx.db.game().iter()
            .filter(|g| g.user == ctx.sender)
            .for_each(|g| {
                ctx.db.game().delete(g);
            });
        log::info!("user {} disconnected", user.name);
    } else {
        log::warn!("unknown user {} disconnected", String::from(ctx.sender.to_abbreviated_hex()));
    }
}