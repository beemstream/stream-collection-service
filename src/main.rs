use dotenv::dotenv;
use futures::future;
use isahc::prelude::*;
use miniserde::{json, Deserialize, Serialize};
use rocket::{
    http::Status,
    request::{FromFormValue, Request as R},
};
use std::{collections::HashMap, fmt::format};
use utils::JsonResponse;

mod utils;
#[macro_use]
extern crate rocket;

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

async fn get_twitch_streams(
    twitch_client_id: String,
    access_token: String,
) -> TwitchStreamsResponse {
    let request = Request::builder()
        .uri("https://api.twitch.tv/helix/streams?game_id=509670&first=100")
        .method("GET")
        .header("Client-ID", twitch_client_id)
        .header("Authorization", format!("Bearer {}", access_token))
        .body(())
        .unwrap();
    let mut response = isahc::send_async(request).await.unwrap();
    let text_response = response.text_async().await.unwrap();
    println!("EXTERNAL REQUEST TWITCH status {:?}", response.status());
    let json_response = json::from_str::<TwitchStreamsResponse>(&text_response);

    match json_response {
        Ok(v) => v,
        Err(e) => panic!("Error Parsing {:?}", e),
    }
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

#[derive(Debug, Deserialize, Serialize)]
struct TwitchPagination {
    cursor: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct TwitchStreamsResponse {
    data: Vec<TwitchStream>,
    pagination: TwitchPagination,
}

fn search_by_category(streams: Vec<TwitchStream>, category_tag: &String) -> Vec<TwitchStream> {
    streams
        .into_iter()
        .filter(|stream| stream.tag_ids.iter().any(|id| id.eq(category_tag)))
        .collect()
}

#[get("/streams?<category>")]
async fn get_streams<'a>(category: Option<Category>) -> JsonResponse<Vec<TwitchStream>> {
    let token = get_twitch_token().await;
    let twitch_client_id: String =
        std::env::var("TWITCH_CLIENT_ID").expect("TWITCH_CLIENT_ID must be set");
    let twitch_tags_map = get_twitch_tag_ids();

    let all_streams = get_twitch_streams(twitch_client_id, token.access_token.clone());

    let streams = match category {
        Some(c) => search_by_category(all_streams.await.data, twitch_tags_map.get(&c).unwrap()),
        None => all_streams.await.data,
    };

    JsonResponse::new(streams, Status::Ok)
}

#[derive(Debug, Eq, PartialEq, Hash, FromFormValue)]
enum Category {
    Programming,
    WebDevelopment,
    GameDevelopment,
    MobileDevelopment,
}

#[catch(404)]
fn not_found(_: &R) -> () {
    ()
}

#[launch]
fn rocket() -> rocket::Rocket {
    dotenv().ok();
    rocket::ignite()
        .mount("/", routes![get_streams])
        .register(catchers![not_found])
}
