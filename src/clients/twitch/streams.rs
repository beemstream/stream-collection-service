use isahc::{AsyncReadResponseExt, Request, Response, AsyncBody};
use rocket::{info, serde::json::serde_json::to_string};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TwitchStream {
    pub game_id: String,
    pub game_name: String,
    pub id: String,
    pub language: String,
    pub started_at: String,
    pub tag_ids: Option<Vec<String>>,
    pub thumbnail_url: String,
    pub title: String,
    pub user_id: String,
    pub user_name: String,
    pub user_login: String,
    pub viewer_count: u64,
    pub r#type: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TwitchPagination {
    pub cursor: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TwitchStreamsResponse {
    pub data: Vec<TwitchStream>,
    pub pagination: TwitchPagination,
}

pub async fn get_science_and_tech_streams(
    twitch_client_id: &str,
    access_token: &str,
    after: &str,
) -> TwitchStreamsResponse {
    let after_query = match after.is_empty() {
        true => after.to_owned(),
        false => format!("&after={}", after),
    };

    let url = format!(
        "https://api.twitch.tv/helix/streams?game_id=509670&first=100{}",
        after_query
    );

    info!("requesting url {}", url);

    let streams = fetch_programming_streams(twitch_client_id, access_token, url).await;

    info!("fetched first stream {:?}", streams.data.get(0));
    info!("fetched {} streams", streams.data.len());

    streams
}

pub async fn get_software_game_dev_streams(
    twitch_client_id: &str,
    access_token: &str,
    after: &str,
) -> TwitchStreamsResponse {
    let after_query = match after.is_empty() {
        true => after.to_owned(),
        false => format!("&after={}", after),
    };

    let url = format!(
        "https://api.twitch.tv/helix/streams?game_id=1469308723&first=100{}",
        after_query
    );

    info!("requesting url {}", url);

    let streams = fetch_programming_streams(twitch_client_id, access_token, url).await;

    info!("fetched first stream {:?}", streams.data.get(0));
    info!("fetched {} streams", streams.data.len());

    streams
}

pub async fn fetch_programming_streams(
    twitch_client_id: &str,
    access_token: &str,
    url: String,
) -> TwitchStreamsResponse {
    let request = Request::builder()
        .uri(url)
        .method("GET")
        .header("Client-ID", twitch_client_id)
        .header("Authorization", format!("Bearer {}", access_token))
        .body(())
        .map_err(|_| TwitchStreamsResponse {
            data: vec![],
            pagination: TwitchPagination { cursor: None },
        })
        .unwrap();

    let body = Response::builder()
        .body(AsyncBody::from(to_string(&TwitchStreamsResponse {
            data: vec![],
            pagination: TwitchPagination { cursor: None },
        }).unwrap()));

    let mut response = isahc::send_async(request)
        .await
        .map_err(|_| body.unwrap())
        .unwrap();

    response
        .json()
        .await
        .unwrap_or_else(|_| TwitchStreamsResponse {
            data: vec![],
            pagination: TwitchPagination { cursor: None },
        })
}
