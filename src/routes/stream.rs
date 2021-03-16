use rocket::{State, http::Status};
use futures::future::join;
use serde::Serialize;

use crate::{states::GlobalConfig, twitch_stream::{TwitchStream, TwitchUser, get_twitch_stream, get_twitch_user}, utils::JsonResponse};


#[derive(Debug, Serialize)]
pub struct StreamDetail {
    stream_info: TwitchStream,
    user_info: TwitchUser
}

#[get("/stream/<username>")]
pub async fn get_stream<'a>(username: String, state: State<'a, GlobalConfig>) -> Result<JsonResponse<StreamDetail>, Status> {
    let token = state.fetch_access_token();

    let twitch_user = get_twitch_user(&state.client_id, &token, &username);
    let twitch_stream = get_twitch_stream(&state.client_id, &token, &username);

    let (user, stream) = join(twitch_user, twitch_stream).await;

    let mut user_data = user?.data;
    info!("user_data got {:?}", user_data);
    let user_info = user_data.swap_remove(0);

    let mut stream_user_data = stream?.data;
    info!("stream_user got {:?}", stream_user_data);
    let stream_info = stream_user_data.swap_remove(0);

    let response = StreamDetail { stream_info, user_info };

    Ok(JsonResponse::new(response, Status::Ok))
}

