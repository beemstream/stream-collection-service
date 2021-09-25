use crate::clients::twitch::streams::TwitchStreamsResponse;
use isahc::{http::StatusCode, AsyncReadResponseExt, Request};
use rocket::{http::Status, info};

pub async fn get_stream(
    twitch_client_id: &str,
    access_token: &str,
    username: &str,
) -> Result<TwitchStreamsResponse, Status> {
    let request = Request::builder()
        .uri(format!(
            "https://api.twitch.tv/helix/streams?user_login={}",
            username
        ))
        .method("GET")
        .header("Client-ID", twitch_client_id)
        .header("Authorization", format!("Bearer {}", access_token))
        .body(())
        .unwrap();

    let mut response = isahc::send_async(request).await.unwrap();

    if response.status() != StatusCode::OK {
        info!("streams not found failed with {:?}", response.status());
        return Err(Status::NotFound);
    }
    Ok(response.json().await.unwrap())
}
