use isahc::{http::StatusCode, AsyncReadResponseExt, Request};
use rocket::{http::Status, info};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct TwitchUserResponse {
    pub data: Vec<TwitchUser>,
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

pub async fn get_user(
    twitch_client_id: &str,
    access_token: &str,
    username: &str,
) -> Result<TwitchUserResponse, Status> {
    let request = Request::builder()
        .uri(format!(
            "https://api.twitch.tv/helix/users?login={}",
            username
        ))
        .method("GET")
        .header("Client-ID", twitch_client_id)
        .header("Authorization", format!("Bearer {}", access_token))
        .body(())
        .unwrap();

    let mut response = isahc::send_async(request).await.unwrap();

    if response.status() != StatusCode::OK {
        info!("user not found failed with {:?}", response.status());
        return Err(Status::NotFound);
    }
    Ok(response.json().await.unwrap())
}
