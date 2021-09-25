use rocket::{catchers, debug, info, launch, routes, Build, Rocket};
use std::sync::{Arc, Mutex};

use catchers::not_found;
use category::get_twitch_tag_ids;
use routes::streams::get_streams;
use routes::{stream::get_stream, streams::fetch_streams_interval};
use states::GlobalConfig;

use crate::catchers::unauthorized;
use crate::clients::twitch::get_all_tags_map;
use crate::routes::follows::get_follows_for_user;

mod catchers;
mod category;
mod clients;
mod guards;
mod routes;
mod states;
mod utils;

#[launch]
async fn start() -> rocket::Rocket<Build> {
    let rocket = Rocket::build();
    let figment = rocket.figment();

    let client_id: String = figment.extract_inner("twitch_client_id").expect("custom");
    let client_secret: String = figment
        .extract_inner("twitch_client_secret")
        .expect("custom");

    let tags = get_twitch_tag_ids();
    let fetched_token = clients::twitch::get_token(&client_id, &client_secret);

    debug!("token fetched at {:?}", fetched_token.access_token);

    let token = Arc::new(Mutex::new(fetched_token.clone()));
    let expires_in = token.lock().unwrap().expires_in;
    let expired = std::time::Duration::from_secs(expires_in);
    let expiring_time = std::time::Instant::now() + expired;

    info!("token expiring at {:?}", expires_in);

    let stream_fetch_interval =
        rocket::tokio::time::interval(rocket::tokio::time::Duration::from_millis(15_000));

    let all_tags = get_all_tags_map(&client_id, &fetched_token.access_token).await;

    rocket::tokio::spawn(fetch_streams_interval(
        stream_fetch_interval,
        client_id.clone(),
        client_secret.clone(),
        fetched_token,
        tags.clone(),
    ));

    let config = GlobalConfig {
        client_id,
        client_secret,
        tags,
        token,
        expired: Arc::new(Mutex::new(expiring_time)),
        all_tags
    };

    rocket
        .mount(
            "/stream-collection",
            routes![get_streams, get_stream, get_follows_for_user],
        )
        .manage(config)
        .register("/", catchers![not_found, unauthorized])
}
