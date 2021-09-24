use crate::{
    clients::twitch::user::{get_user_follows, TwitchUserFollow},
    guards::twitch_auth::AccessTokenResponse,
    routes::streams::STREAMS_CACHE,
    states::GlobalConfig,
};
use rocket::{get, info, serde::json::Json, State};

#[get("/follows")]
pub async fn get_follows_for_user(
    state: &State<GlobalConfig>,
    access_token: AccessTokenResponse,
) -> Json<Vec<TwitchUserFollow>> {
    let parsed_token = access_token.token.split(' ').collect::<Vec<&str>>();

    let mut all_follows = get_user_follows(
        &state.client_id,
        &parsed_token[0],
        &access_token.validate_token.user_id,
        "",
    )
    .await;
    let mut cursor = all_follows.pagination.cursor;

    while cursor.is_some() {
        info!("get_twitch_user_follows: fetching cursor {:?}", cursor);

        let mut stream_response = get_user_follows(
            &state.client_id,
            &parsed_token[0],
            &access_token.validate_token.user_id,
            cursor.unwrap().as_str(),
        )
        .await;

        info!(
            "get_twitch_user_follows: got {} follows",
            stream_response.data.len()
        );

        all_follows.data.append(&mut stream_response.data);

        cursor = stream_response.pagination.cursor;
    }

    let data = all_follows.data;

    let streams = STREAMS_CACHE.lock().unwrap();

    let follows = data
        .into_iter()
        .filter(|d| streams.iter().any(|s| s.user_id == d.to_id))
        .collect();

    Json(follows)
}
