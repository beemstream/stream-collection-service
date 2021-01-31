use rocket::{State, http::Status};
use futures::future::join;
use serde::Serialize;

use crate::{states::GlobalConfig, twitch_stream::{TwitchStream, TwitchUser, get_twitch_stream, get_twitch_user}, utils::JsonResponse};


#[derive(Debug, Serialize)]
pub struct StreamDetail {
    stream_info: TwitchStream,
    user_info: TwitchUser
}

#[get("/stream/<id>")]
pub async fn get_stream<'a>(id: String, state: State<'a, GlobalConfig>) -> Result<JsonResponse<StreamDetail>, Status> {
    let token = state.fetch_access_token();

    let twitch_user = get_twitch_user(&state.client_id, &token, &id);
    let twitch_stream = get_twitch_stream(&state.client_id, &token, &id);

    let (user, stream) = join(twitch_user, twitch_stream).await;

    let user_info = user?.data.swap_remove(0);
    let stream_info = stream?.data.swap_remove(0);

    let response = StreamDetail { stream_info, user_info };

    Ok(JsonResponse::new(response, Status::Ok))
}

