use isahc::{AsyncReadResponseExt, Request, http::StatusCode};
use rocket::http::Status;
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
    let after_query = match after.is_empty() {
        true => after.to_owned(),
        false => format!("&after={}", after)
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

#[derive(Debug, Deserialize, Serialize)]
pub struct TwitchUserResponse {
    pub data: Vec<TwitchUser>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TwitchUser {
    pub id: String,
    pub login: String,
    pub display_name: String,
    pub r#type: String,
    pub broadcaster_type: String,
    pub description: String,
    pub profile_image_url: String,
    pub offline_image_url: String,
    pub view_count: u32,
}

pub async fn get_twitch_user(
    twitch_client_id: &String,
    access_token: &String,
    id: &String,
) -> Result<TwitchUserResponse, Status> {

    let request = Request::builder()
        .uri(format!("https://api.twitch.tv/helix/users?id={}", id))
        .method("GET")
        .header("Client-ID", twitch_client_id)
        .header("Authorization", format!("Bearer {}", access_token))
        .body(())
        .unwrap();

    let mut response = isahc::send_async(request).await.unwrap();

    if response.status() != StatusCode::OK {
        return Err(Status::NotFound)
    }
    Ok(response.json().await.unwrap())
}

pub async fn get_twitch_stream(
    twitch_client_id: &String,
    access_token: &String,
    id: &String,
) -> Result<TwitchStreamsResponse, Status> {

    let request = Request::builder()
        .uri(format!("https://api.twitch.tv/helix/streams?user_id={}", id))
        .method("GET")
        .header("Client-ID", twitch_client_id)
        .header("Authorization", format!("Bearer {}", access_token))
        .body(())
        .unwrap();

    let mut response = isahc::send_async(request).await.unwrap();

    if response.status() != StatusCode::OK {
        return Err(Status::NotFound)
    }
    Ok(response.json().await.unwrap())
}
