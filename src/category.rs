use rocket::FromFormField;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq, Hash, FromFormField, Serialize, Clone)]
pub enum Category {
    Programming,
    WebDevelopment,
    GameDevelopment,
    MobileDevelopment,
}

// static
pub fn get_twitch_tag_ids() -> HashMap<Category, String> {
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
