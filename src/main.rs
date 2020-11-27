use dotenv::dotenv;
use isahc::prelude::*;
use miniserde::{json, Deserialize, Serialize};
use std::collections::HashMap;
use futures::future;

#[macro_use]
extern crate rocket;

#[derive(Eq, PartialEq, Hash)]
enum Category {
    Programming,
    WebDevelopment,
    GameDevelopment,
    MobileDevelopment,
}

// static
fn get_twitch_tag_ids() -> HashMap<Category, String> {
    let mut hash: HashMap<Category, String> = HashMap::new();
    hash.insert(
        Category::Programming,
        "a59f1e4e-257b-4bd0-90c7-189c3efbf917".to_owned(),
    );
    hash.insert(
        Category::WebDevelopment,
        "c23ce252-cf78-4b98-8c11-8769801aaf3a".to_owned(),
    );
    hash.insert(
        Category::GameDevelopment,
        "f588bd74-e496-4d11-9169-3597f38a5d25".to_owned(),
    );
    hash.insert(
        Category::MobileDevelopment,
        "6e23d976-33ec-47e8-b22b-3727acd41862".to_owned(),
    );
    hash
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct User {
    userId: u32,
    id: u32,
    title: String,
    completed: bool,
}

#[derive(Debug, Deserialize)]
struct Token {
    access_token: String,
    expires_in: u64,
    token_type: String,
}

async fn get_twitch_token() -> Token {
    let twitch_client_id: String =
        std::env::var("TWITCH_CLIENT_ID").expect("TWITCH_CLIENT_ID must be set");
    let twitch_client_secret: String =
        std::env::var("TWITCH_CLIENT_SECRET").expect("TWITCH_CLIENT_ID must be set");
    let url = format!(
        "https://id.twitch.tv/oauth2/token?client_id={}&client_secret={}&grant_type={}",
        twitch_client_id, twitch_client_secret, "client_credentials"
    );
    let mut response = isahc::post_async(url, "").await.unwrap();
    let text_response = response.text_async().await.unwrap();
    json::from_str::<Token>(&text_response).unwrap()
}

async fn get_twitch_streams_offset(offset: i16, twitch_client_id: String, access_token: String) -> TwitchStreamsResponse {
    let request = Request::builder()
        .uri("https://api.twitch.tv/helix/streams?game_id=509670&first=100")
        .method("GET")
        .header("Client-ID", twitch_client_id)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("offset", offset)
        .body(())
        .unwrap();
    let mut response = isahc::send_async(request).await.unwrap();
    let text_response = response.text_async().await.unwrap();
    json::from_str::<TwitchStreamsResponse>(&text_response).unwrap()
}

#[derive(Debug, Deserialize, Serialize)]
struct TwitchStream {
    game_id: String,
    id: String,
    language: String,
    started_at: String,
    tag_ids: Vec<String>,
    thumbnail_url: String,
    title: String,
    user_id: String,
    user_name: String,
    viewer_count: u32,
}

#[derive(Debug, Deserialize)]
struct TwitchPagination {
    cursor: String,
}

#[derive(Debug, Deserialize)]
struct TwitchStreamsResponse {
    data: Vec<TwitchStream>,
    pagination: TwitchPagination,
}

#[get("/streams")]
async fn get_streams<'a>() -> String {
    let token = get_twitch_token().await;
    let twitch_client_id: String =
        std::env::var("TWITCH_CLIENT_ID").expect("TWITCH_CLIENT_ID must be set");

    let mut requests = vec![];
    for i in 0..11 {
        requests.push(get_twitch_streams_offset(
            i,
            twitch_client_id.clone(),
            token.access_token.clone(),
        ));
    }
    let every_response = future::join_all(requests).await;

    let last = every_response.last().unwrap();
    println!("has streams remaining {:?}", last.pagination.cursor);
    let all_streams: Vec<TwitchStream> = every_response.into_iter()
        .flat_map(|s| s.data)
        .collect();

    json::to_string(&all_streams)
}

#[launch]
fn rocket() -> rocket::Rocket {
    dotenv().ok();
    rocket::ignite().mount("/", routes![get_streams])
}
