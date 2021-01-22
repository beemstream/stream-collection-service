use isahc::{ReadResponseExt, Request, AsyncReadResponseExt};
use rocket::{Rocket, State, http::Status, request::{FromFormValue, Request as R}};
use std::{collections::HashMap, sync::{Arc, Mutex}};
use utils::JsonResponse;
use serde::{Deserialize, Serialize};

mod utils;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate log;

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

// static
fn get_twitch_categories() -> HashMap<String, Category> {
    let mut hash: HashMap<String, Category> = HashMap::new();
    hash.insert(
        "a59f1e4e-257b-4bd0-90c7-189c3efbf917".to_owned(),
        Category::Programming,
    );
    hash.insert(
        "c23ce252-cf78-4b98-8c11-8769801aaf3a".to_owned(),
        Category::WebDevelopment,
    );
    hash.insert(
        "f588bd74-e496-4d11-9169-3597f38a5d25".to_owned(),
        Category::GameDevelopment,
    );
    hash.insert(
        "6e23d976-33ec-47e8-b22b-3727acd41862".to_owned(),
        Category::MobileDevelopment,
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

#[derive(Debug, Deserialize, Clone)]
struct Token {
    access_token: String,
    expires_in: u64,
    token_type: String,
}

fn get_twitch_token(client_id: &String, client_secret: &String) -> Token {
    let url = format!(
        "https://id.twitch.tv/oauth2/token?client_id={}&client_secret={}&grant_type={}",
        client_id, client_secret, "client_credentials"
    );

    let mut response = isahc::post(url, "").unwrap();
    response.json().unwrap()
}

async fn get_twitch_streams(
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

#[derive(Debug, Deserialize, Serialize, Clone)]
struct TwitchStream {
    game_id: String,
    game_name: String,
    id: String,
    language: String,
    started_at: String,
    tag_ids: Option<Vec<String>>,
    thumbnail_url: String,
    title: String,
    user_id: String,
    user_name: String,
    viewer_count: u64,
    r#type: String
}

#[derive(Debug, Deserialize, Serialize)]
struct TwitchPagination {
    cursor: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct TwitchStreamsResponse {
    data: Vec<TwitchStream>,
    pagination: TwitchPagination,
}

fn filter_by_category(streams: Vec<TwitchStream>, category_tag: &String, categories: &HashMap<String, Category>) -> Vec<TwitchStream> {
    streams
        .into_iter()
        .filter(|stream| {
            match &stream.tag_ids {
                Some(tags) => tags.iter().any(|id| id.eq(category_tag)),
                None => false
            }
        })
    .collect()
}


fn filter_all_programming_streams<'a>(streams: Vec<TwitchStream>, tag_ids: &HashMap<Category, String>, categories: &HashMap<String, Category>) -> Vec<TwitchStream> {
    let tag_id_vals: Vec<&String> = tag_ids.values().collect();
    streams
        .into_iter()
        .filter(|stream| {
            match &stream.tag_ids {
                Some(tags) => tags.iter().any(|id| tag_id_vals.contains(&id)),
                None => false
            }
        })
        .collect()
}

#[get("/streams?<category>")]
async fn get_streams<'a>(
    state: State<'a, GlobalConfig>,
    category: Option<Category>,
) -> JsonResponse<Vec<TwitchStream>> {
    let token = state.fetch_access_token();
    let categories = &state.categories;
    let mut all_streams = get_twitch_streams(&state.client_id, &token, "").await;
    let mut cursor = all_streams.pagination.cursor;

    while cursor.is_some() {
        info!("fetching cursor {:?}", cursor);

        let mut stream_response = get_twitch_streams(&state.client_id, &token, cursor.unwrap().as_str()).await;

        info!("got {} streams", stream_response.data.len());

        all_streams.data.append(&mut stream_response.data);

        cursor = stream_response.pagination.cursor;
    }

    let data = all_streams.data;

    let streams = match category {
        Some(c) => filter_by_category(data, state.tags.get(&c).unwrap(), categories),
        None => filter_all_programming_streams(data, &state.tags, categories),
    };

    JsonResponse::new(streams, Status::Ok)
}

#[derive(Debug, Eq, PartialEq, Hash, FromFormValue, Serialize)]
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

struct GlobalConfig {
    client_id: String,
    client_secret: String,
    tags: HashMap<Category, String>,
    categories: HashMap<String, Category>,
    token: Arc<Mutex<Token>>,
    expired: Arc<Mutex<std::time::Instant>>
}

impl GlobalConfig {
    pub fn fetch_access_token(&self) -> String {
        let is_expired = std::time::Instant::now() >= self.expired.lock().unwrap().clone();

        if is_expired {
            info!("token expired at: {:?}", std::time::Instant::now());
            let token_response = get_twitch_token(&self.client_id, &self.client_secret);
            *self.expired.lock().unwrap() = std::time::Instant::now() + std::time::Duration::from_secs(token_response.expires_in);
            *self.token.lock().unwrap() = token_response;
        }
        self.token.lock().unwrap().access_token.clone()
    }
}

#[launch]
async fn rocket() -> rocket::Rocket {
    env_logger::init();
    let rocket = Rocket::ignite();
    let figment = rocket.figment();

    let client_id: String = figment.extract_inner("twitch_client_id").expect("custom");
    let client_secret: String = figment.extract_inner("twitch_client_secret").expect("custom");

    let tags = get_twitch_tag_ids();
    let categories = get_twitch_categories();
    let fetched_token = get_twitch_token(&client_id, &client_secret);

    debug!("token fetched at {:?}", fetched_token.access_token);

    let token = Arc::new(Mutex::new(fetched_token));
    let expires_in = token.lock().unwrap().expires_in.clone();
    let expired = std::time::Duration::from_secs(expires_in);
    let expiring_time = std::time::Instant::now() + expired;

    info!("token expiring at {:?}", expires_in);

    let config = GlobalConfig {
        client_id,
        client_secret,
        categories,
        tags,
        token,
        expired: Arc::new(Mutex::new(expiring_time))
    };

    rocket
        .mount("/", routes![get_streams])
        .manage(config)
        .register(catchers![not_found])
}
