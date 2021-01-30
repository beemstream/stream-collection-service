use rocket::{Rocket, State, http::Status};
use tags::get_twitch_tag_names;
use std::sync::{Arc, Mutex};

use catchers::not_found;
use category::{Category, get_twitch_categories, get_twitch_tag_ids};
use states::GlobalConfig;
use utils::{JsonResponse, filter_all_programming_streams, filter_by_category};
use twitch_stream::{TwitchStream, get_twitch_streams};
use twitch_token::get_twitch_token;

mod utils;
mod twitch_stream;
mod twitch_token;
mod category;
mod states;
mod catchers;
mod tags;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate log;

#[get("/streams?<category>")]
async fn get_streams<'a>(
    state: State<'a, GlobalConfig>,
    category: Option<Category>,
) -> JsonResponse<Vec<TwitchStream>> {
    let token = state.fetch_access_token();
    let mut all_streams = get_twitch_streams(&state.client_id, &token, "").await;
    let mut cursor = all_streams.pagination.cursor;

    while cursor.is_some() {
        info!("get_twitch_streams: fetching cursor {:?}", cursor);

        let mut stream_response = get_twitch_streams(&state.client_id, &token, cursor.unwrap().as_str()).await;

        info!("get_twitch_streams: got {} streams", stream_response.data.len());

        all_streams.data.append(&mut stream_response.data);

        cursor = stream_response.pagination.cursor;
    }

    let data = all_streams.data;

    let streams = match category {
        Some(c) => filter_by_category(data, state.tags.get(&c).unwrap(), &state.tags),
        None => filter_all_programming_streams(data, &state.tags),
    };

    JsonResponse::new(streams, Status::Ok)
}

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
        .mount("/", routes![get_streams])
        .manage(config)
        .register(catchers![not_found])
}
