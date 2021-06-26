use crate::{
    category::Category,
    states::GlobalConfig,
    twitch_stream::{self, TwitchStream},
    twitch_token::{self, Token},
    utils::{filter_all_programming_streams, filter_by_category, JsonResponse},
};
use futures::{future::BoxFuture, FutureExt};
use once_cell::sync::Lazy;
use rocket::{get, http::Status, info, tokio::time::Interval, State};
use std::{collections::HashMap, sync::Mutex};

pub static ARRAY: Lazy<Mutex<Vec<TwitchStream>>> = Lazy::new(|| Mutex::new(vec![]));

pub fn fetch_access_token(token: Token, client_id: &str, client_secret: &str) -> Token {
    let expired = std::time::Duration::from_secs(token.expires_in);
    let expiring_time = std::time::Instant::now() + expired;
    let is_expired = std::time::Instant::now() >= expiring_time;

    if is_expired {
        info!("token expired at: {:?}", std::time::Instant::now());
        twitch_token::get_twitch_token(&client_id, &client_secret)
    } else {
        token
    }
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

        let mut all_streams =
            twitch_stream::get_twitch_streams(&client_id, &access_token, "").await;
        let mut cursor = all_streams.pagination.cursor;

        while cursor.is_some() {
            info!("get_twitch_streams: fetching cursor {:?}", cursor);

            let mut stream_response = twitch_stream::get_twitch_streams(
                &client_id,
                &access_token,
                cursor.unwrap().as_str(),
            )
            .await;

            info!(
                "get_twitch_streams: got {} streams",
                stream_response.data.len()
            );

            all_streams.data.append(&mut stream_response.data);

            cursor = stream_response.pagination.cursor;
        }

        let data = all_streams.data;

        *ARRAY.lock().unwrap() = data;

        fetch_streams_interval(interval, client_id, client_secret, token, tags).await
    }
    .boxed()
}

#[get("/streams?<category>")]
pub async fn get_streams(
    state: &State<GlobalConfig>,
    category: Option<Category>,
) -> JsonResponse<Vec<TwitchStream>> {
    let data = ARRAY.lock().unwrap().clone();

    info!("got category {:?}", category);

    let streams = match category {
        Some(c) => filter_by_category(data, state.tags.get(&c).unwrap(), &state.tags),
        None => filter_all_programming_streams(data, &state.tags),
    };

    JsonResponse::new(streams, Status::Ok)
}
