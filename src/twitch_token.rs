use isahc::ReadResponseExt;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Token {
    pub access_token: String,
    pub expires_in: u64,
    pub token_type: String,
}

pub fn get_twitch_token(client_id: &String, client_secret: &String) -> Token {
    let url = format!(
        "https://id.twitch.tv/oauth2/token?client_id={}&client_secret={}&grant_type={}",
        client_id, client_secret, "client_credentials"
    );

    let mut response = isahc::post(url, "").unwrap();
    response.json().unwrap()
}


