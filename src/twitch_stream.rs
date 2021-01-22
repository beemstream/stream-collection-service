use isahc::{Request, AsyncReadResponseExt};
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
    pub viewer_count: u64,
    pub r#type: String
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

pub async fn get_twitch_streams(
    twitch_client_id: &String,
    access_token: &String,
    after: &str,
) -> TwitchStreamsResponse {
    let after_query = {
        if !after.is_empty() {
            format!("&after={}", after)
        } else {
            after.to_owned()
        }
    };

    let request = Request::builder()
        .uri(format!("https://api.twitch.tv/helix/streams?game_id=509670&first=100{}", after_query))
        .method("GET")
        .header("Client-ID", twitch_client_id)
        .header("Authorization", format!("Bearer {}", access_token))
        .body(())
        .unwrap();

    let mut response = isahc::send_async(request).await.unwrap();

    response.json().await.unwrap()
}
