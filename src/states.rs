use crate::{
    category::Category,
    clients::twitch::{get_token, Token},
};
use rocket::info;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub struct GlobalConfig {
    pub client_id: String,
    pub client_secret: String,
    pub tags: HashMap<Category, String>,
    pub token: Arc<Mutex<Token>>,
    pub expired: Arc<Mutex<std::time::Instant>>,
    pub all_tags: HashMap<String, String>,
}

impl GlobalConfig {
    pub fn fetch_access_token(&self) -> String {
        let is_expired = std::time::Instant::now() >= *self.expired.lock().unwrap();

        if is_expired {
            info!("token expired at: {:?}", std::time::Instant::now());
            let token_response = get_token(&self.client_id, &self.client_secret);
            *self.expired.lock().unwrap() = std::time::Instant::now()
                + std::time::Duration::from_secs(token_response.expires_in);
            *self.token.lock().unwrap() = token_response;
        }

        self.token.lock().unwrap().access_token.clone()
    }
}
