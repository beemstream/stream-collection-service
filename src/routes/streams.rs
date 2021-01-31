use rocket::{State, http::Status};

use crate::{category::Category, states::GlobalConfig, twitch_stream::{TwitchStream, get_twitch_streams}, utils::{JsonResponse, filter_all_programming_streams, filter_by_category}};


#[get("/streams?<category>")]
pub async fn get_streams<'a>(
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

