use futures::future::join;
use rocket::{get, http::Status, info, State};
use serde::Serialize;

use crate::{
    clients::twitch::{
        user::{self, TwitchUser},
        TwitchStream,
    },
    states::GlobalConfig,
    utils::JsonResponse,
};

#[derive(Debug, Serialize)]
pub struct StreamDetail {
    #[serde(skip_serializing_if = "Option::is_none")]
    stream_info: Option<TwitchStream>,
    user_info: Option<TwitchUser>,
}

#[get("/stream/<username>")]
pub async fn get_stream(
    username: String,
    state: &State<GlobalConfig>,
) -> Result<JsonResponse<StreamDetail>, Status> {
    let token = state.fetch_access_token();

    let twitch_user = user::get_user(&state.client_id, &token, &username);
    let twitch_stream = user::get_stream(&state.client_id, &token, &username);

    let (user, stream) = join(twitch_user, twitch_stream).await;

    let mut user_data = user?.data;
    info!("user_data got {:?}", user_data);

    let user_info = match user_data.len() {
        1 => Some(user_data.swap_remove(0)),
        _ => return Err(Status::NotFound),
    };

    let mut stream_user_data = stream?.data;
    info!("stream_user got {:?}", stream_user_data);

    let stream_info = match stream_user_data.len() {
        1 => Some(stream_user_data.swap_remove(0)),
        _ => None,
    };

    let response = StreamDetail {
        stream_info,
        user_info,
    };

    Ok(JsonResponse::new(response, Status::Ok))
}
