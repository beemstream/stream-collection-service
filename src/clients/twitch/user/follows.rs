use isahc::{AsyncReadResponseExt, Request};
use serde::{Deserialize, Serialize};

use crate::clients::twitch::TwitchPagination;

#[derive(Debug, Deserialize, Serialize)]
pub struct TwitchUserFollow {
    pub from_id: String,
    pub from_login: String,
    pub from_name: String,
    pub to_id: String,
    pub to_name: String,
    pub followed_at: String,
}

#[derive(Debug, Deserialize)]
pub struct TwitchUserFollows {
    pub data: Vec<TwitchUserFollow>,
    pub pagination: TwitchPagination,
}

pub async fn get_user_follows(
    twitch_client_id: &str,
    access_token: &str,
    user_id: &str,
    after: &str,
) -> TwitchUserFollows {
    let after_query = match after.is_empty() {
        true => after.to_owned(),
        false => format!("&after={}", after),
    };

    let request = Request::builder()
        .uri(format!(
            "https://api.twitch.tv/helix/users/follows?from_id={}&first=100&after={}",
            user_id, after_query
        ))
        .method("GET")
        .header("Client-ID", twitch_client_id)
        .header("Authorization", format!("Bearer {}", access_token))
        .body(())
        .map_err(|_| TwitchUserFollows {
            data: vec![],
            pagination: TwitchPagination { cursor: None },
        })
        .unwrap();

    let mut response = isahc::send_async(request)
        .await
        .map_err(|_| TwitchUserFollows {
            data: vec![],
            pagination: TwitchPagination { cursor: None },
        })
        .unwrap();

    response.json().await.unwrap_or_else(|_| TwitchUserFollows {
        data: vec![],
        pagination: TwitchPagination { cursor: None },
    })
}
