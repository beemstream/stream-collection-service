use isahc::AsyncReadResponseExt;
use rocket::request::{self, FromRequest, Request};
use rocket::{http::Status, outcome::Outcome};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TwitchValidateToken {
    pub client_id: String,
    pub login: String,
    pub scopes: Vec<String>,
    pub user_id: String,
    pub expires_in: u32,
}

pub struct AccessTokenResponse {
    pub validate_token: TwitchValidateToken,
    pub token: String,
}

pub type TokenResponse = Outcome<AccessTokenResponse, (Status, AccessTokenError), ()>;

pub async fn authenticate_twitch_user(access_token: &str) -> Result<TokenResponse, TokenResponse> {
    let request = isahc::Request::builder()
        .uri("https://id.twitch.tv/oauth2/validate")
        .method("GET")
        .header("Authorization", format!("OAuth {}", access_token))
        .body(())
        .map_err(|_| {
            Outcome::<AccessTokenResponse, (Status, AccessTokenError), ()>::Failure((
                Status::Unauthorized,
                AccessTokenError::Invalid,
            ))
        })?;

    let mut response = isahc::send_async(request).await.map_err(|_| {
        Outcome::<AccessTokenResponse, (Status, AccessTokenError), ()>::Failure((
            Status::Unauthorized,
            AccessTokenError::Invalid,
        ))
    })?;

    let json_response = response.json::<TwitchValidateToken>().await.map_err(|_| {
        Outcome::<AccessTokenResponse, (Status, AccessTokenError), ()>::Failure((
            Status::Unauthorized,
            AccessTokenError::Invalid,
        ))
    })?;

    Ok(Outcome::Success(AccessTokenResponse {
        validate_token: json_response,
        token: access_token.to_string(),
    }))
}

pub fn is_token_valid(token: &str) -> bool {
    let request_token: Vec<&str> = token.split(' ').collect();
    request_token.starts_with(&["Bearer"])
}
#[derive(Debug)]
pub struct AccessToken(pub String);

#[derive(Debug)]
pub enum AccessTokenError {
    Missing,
    Invalid,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AccessTokenResponse {
    type Error = AccessTokenError;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let keys = req.headers().get("token").collect::<Vec<&str>>();

        match keys.len() {
            0 => Outcome::Failure((Status::Unauthorized, AccessTokenError::Missing)),
            1 if is_token_valid(keys[0]) => {
                let request_token: Vec<&str> = keys[0].split(' ').collect();
                match authenticate_twitch_user(request_token[1]).await {
                    Ok(v) => v,
                    Err(e) => e,
                }
            }
            _ => Outcome::Failure((Status::Unauthorized, AccessTokenError::Invalid)),
        }
    }
}
