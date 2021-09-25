use crate::{category::Category, clients::twitch::{Token, TwitchStream, TwitchStreamsResponse, get_science_and_tech_streams, get_software_game_dev_streams, get_token}, states::GlobalConfig, utils::{filter_all_programming_streams, filter_by_category, JsonResponse}};
use futures::{future::BoxFuture, FutureExt};
use once_cell::sync::Lazy;
use rocket::{
    get,
    http::Status,
    info,
    tokio::{join, time::Interval},
    State,
};
use std::{collections::HashMap, sync::Mutex};

pub static STREAMS_CACHE: Lazy<Mutex<Vec<TwitchStream>>> = Lazy::new(|| Mutex::new(vec![]));

pub fn fetch_access_token(token: Token, client_id: &str, client_secret: &str) -> Token {
    let expired = std::time::Duration::from_secs(token.expires_in);
    let expiring_time = std::time::Instant::now() + expired;
    let is_expired = std::time::Instant::now() >= expiring_time;

    if is_expired {
        info!("token expired at: {:?}", std::time::Instant::now());
        get_token(client_id, client_secret)
    } else {
        token
    }
}

pub enum TwitchCategory {
    ScienceAndTechnology,
    SoftwareAndGameDevelopment,
}

pub async fn fetch_all_livestreams(
    mut all_streams: TwitchStreamsResponse,
    client_id: &str,
    access_token: &str,
    stream_source: TwitchCategory,
) -> TwitchStreamsResponse {
    let mut cursor = all_streams.pagination.cursor.clone();

    info!(
        "fetch_all_livestreams: starting total {}",
        all_streams.data.len()
    );
    while cursor.is_some() {
        info!("fetch_all_livestreams: fetching cursor {:?}", cursor);

        let mut stream_response = match stream_source {
            TwitchCategory::ScienceAndTechnology => {
                get_science_and_tech_streams(client_id, access_token, cursor.unwrap().as_str())
                    .await
            }
            TwitchCategory::SoftwareAndGameDevelopment => {
                get_software_game_dev_streams(client_id, access_token, cursor.unwrap().as_str())
                    .await
            }
        };

        all_streams.data.append(&mut stream_response.data);

        info!(
            "fetch_all_livestreams: got {} streams",
            all_streams.data.len()
        );

        cursor = stream_response.pagination.cursor;
    }

    all_streams
}

pub fn fetch_streams_interval(
    mut interval: Interval,
    client_id: String,
    client_secret: String,
    token: Token,
    tags: HashMap<Category, String>,
) -> BoxFuture<'static, ()> {
    async move {
        interval.tick().await;

        let access_token =
            fetch_access_token(token.clone(), &client_id, &client_secret).access_token;

        let science_and_tech_stream_handle = fetch_all_livestreams(
            get_science_and_tech_streams(&client_id, &access_token, "").await,
            &client_id,
            &access_token,
            TwitchCategory::ScienceAndTechnology,
        );

        let software_and_game_dev_streams_handle = fetch_all_livestreams(
            get_software_game_dev_streams(&client_id, &access_token, "").await,
            &client_id,
            &access_token,
            TwitchCategory::SoftwareAndGameDevelopment,
        );

        let (science_and_tech_streams, mut software_and_game_dev_streams) = join!(
            science_and_tech_stream_handle,
            software_and_game_dev_streams_handle
        );

        let mut data = science_and_tech_streams.data;

        data.append(&mut software_and_game_dev_streams.data);

        data.sort_by(|a, b| b.viewer_count.cmp(&a.viewer_count));

        *STREAMS_CACHE.lock().unwrap() = data;

        fetch_streams_interval(interval, client_id, client_secret, token, tags).await
    }
    .boxed()
}

#[get("/streams?<category>")]
pub async fn get_streams(
    state: &State<GlobalConfig>,
    category: Option<Category>,
) -> JsonResponse<Vec<TwitchStream>> {
    let data = STREAMS_CACHE.lock().unwrap().clone();

    info!("got category {:?}", category);

    let streams = match category {
        Some(c) => filter_by_category(data, state.tags.get(&c).unwrap(), &state.all_tags),
        None => filter_all_programming_streams(data, &state.tags, &state.all_tags),
    };

    JsonResponse::new(streams, Status::Ok)
}
