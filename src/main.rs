use rocket::{catchers, debug, info, launch, routes, Build, Rocket};
use std::sync::{Arc, Mutex};

use catchers::not_found;
use category::{get_twitch_categories, get_twitch_tag_ids};
use routes::streams::get_streams;
use routes::{stream::get_stream, streams::fetch_streams_interval};
use states::GlobalConfig;
use twitch_token::get_twitch_token;

mod catchers;
mod category;
mod routes;
mod states;
mod tags;
mod twitch_stream;
mod twitch_token;
mod utils;

#[launch]
async fn start() -> rocket::Rocket<Build> {
    openssl_probe::init_ssl_cert_env_vars();
    let rocket = Rocket::build();
    let figment = rocket.figment();

    let client_id: String = figment.extract_inner("twitch_client_id").expect("custom");
    let client_secret: String = figment
        .extract_inner("twitch_client_secret")
        .expect("custom");

    let tags = get_twitch_tag_ids();
    let categories = get_twitch_categories();
    let fetched_token = get_twitch_token(&client_id, &client_secret);

    debug!("token fetched at {:?}", fetched_token.access_token);

    let token = Arc::new(Mutex::new(fetched_token.clone()));
    let expires_in = token.lock().unwrap().expires_in;
    let expired = std::time::Duration::from_secs(expires_in);
    let expiring_time = std::time::Instant::now() + expired;

    info!("token expiring at {:?}", expires_in);

    let stream_fetch_interval =
        rocket::tokio::time::interval(rocket::tokio::time::Duration::from_millis(15_000));

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
        categories,
        tags,
        token,
        expired: Arc::new(Mutex::new(expiring_time)),
    };

    rocket
        .mount("/stream-collection", routes![get_streams, get_stream])
        .manage(config)
        .register("/", catchers![not_found])
}
