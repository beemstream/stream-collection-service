use std::collections::HashMap;

use isahc::{AsyncReadResponseExt, Request};
use rocket::info;
use serde::{Deserialize, Serialize};

use super::TwitchPagination;

#[derive(Debug, Deserialize, Serialize)]
pub struct Localization {
    #[serde(rename="en-us")]
    pub en_us: String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TwitchTag {
    pub tag_id: String,
    pub localization_names: Localization
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TagResponse {
    pub data: Vec<TwitchTag>,
    pub pagination: TwitchPagination,
}

pub async fn get_all_tags(
    twitch_client_id: &str,
    access_token: &str,
    after: &str,
) -> TagResponse {
    let after_query = match after.is_empty() {
        true => after.to_owned(),
        false => format!("&after={}", after),
    };

    let url = format!("https://api.twitch.tv/helix/tags/streams?first=100{}", after_query);

    info!("requesting url {}", url);

    let request = Request::builder()
        .uri(url)
        .method("GET")
        .header("Client-ID", twitch_client_id)
        .header("Authorization", format!("Bearer {}", access_token))
        .body(())
        .unwrap();

    let mut response = isahc::send_async(request)
        .await.unwrap();

    println!("{:?}", response.text().await);

    response.json().await.unwrap()
}

pub async fn get_all_tags_map(
    client_id: &str,
    access_token: &str,
) -> HashMap<String, String> {
    let all_tags = get_all_tags(client_id, access_token, "").await;
    let mut cursor = all_tags.pagination.cursor.clone();

    let mut all_tags_map = all_tags.data.into_iter().fold(HashMap::new(), |mut acc, curr| {
        acc.insert(curr.tag_id, curr.localization_names.en_us.to_lowercase().replace("development", "dev"));
        acc
    });

    info!(
        "fetch_all_tags: starting total {}",
        all_tags_map.keys().len()
    );
    while cursor.is_some() {
        info!("fetch_all_tags: fetching cursor {:?}", cursor);

        let tags_response = get_all_tags(client_id, access_token, cursor.unwrap().as_str()).await;

        let map = tags_response.data.into_iter().fold(HashMap::new(), |mut acc, curr| {
            acc.insert(curr.tag_id, curr.localization_names.en_us.to_lowercase().replace("development", "dev"));
            acc
        });

        all_tags_map.extend(map);

        info!(
            "fetch_all_tags: got {} tags",
            all_tags_map.keys().len()
        );

        cursor = tags_response.pagination.cursor;
    }

    info!("ALL TAGS {:#?}", all_tags_map);

    all_tags_map
}

