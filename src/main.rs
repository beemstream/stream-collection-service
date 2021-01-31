use rocket::Rocket;
use std::sync::{Arc, Mutex};

use catchers::not_found;
use category::{get_twitch_categories, get_twitch_tag_ids};
use states::GlobalConfig;
use twitch_token::get_twitch_token;
use routes::stream::get_stream;
use routes::streams::get_streams;

mod utils;
mod twitch_stream;
mod twitch_token;
mod category;
mod states;
mod catchers;
mod tags;
mod routes;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate log;

#[launch]
async fn start() -> rocket::Rocket {
    openssl_probe::init_ssl_cert_env_vars();
    env_logger::init();
    let rocket = Rocket::ignite();
    let figment = rocket.figment();

    let client_id: String = figment.extract_inner("twitch_client_id").expect("custom");
    let client_secret: String = figment.extract_inner("twitch_client_secret").expect("custom");

    let tags = get_twitch_tag_ids();
    let categories = get_twitch_categories();
    let fetched_token = get_twitch_token(&client_id, &client_secret);

    debug!("token fetched at {:?}", fetched_token.access_token);

    let token = Arc::new(Mutex::new(fetched_token));
    let expires_in = token.lock().unwrap().expires_in.clone();
    let expired = std::time::Duration::from_secs(expires_in);
    let expiring_time = std::time::Instant::now() + expired;

    info!("token expiring at {:?}", expires_in);

    let config = GlobalConfig {
        client_id,
        client_secret,
        categories,
        tags,
        token,
        expired: Arc::new(Mutex::new(expiring_time))
    };

    rocket
        .mount("/", routes![get_streams, get_stream])
        .manage(config)
        .register(catchers![not_found])
}
