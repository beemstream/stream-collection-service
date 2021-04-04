use serde::Deserialize;
use std::collections::HashMap;

use crate::category::Category;

#[derive(Deserialize, Debug)]
pub struct TwitchTag {
    data: Vec<LocalizedData>,
}

#[derive(Deserialize, Debug)]
pub struct LocalizedData {
    localization_names: LocalizedName,
}

#[derive(Deserialize, Debug)]
pub struct LocalizedName {
    #[serde(rename(deserialize = "en-us"))]
    en_us: String,
}

pub fn get_twitch_tag_names(
    tag_ids: Vec<String>,
    categories: &HashMap<Category, String>,
) -> Vec<String> {
    tag_ids
        .into_iter()
        .filter(|tag| {
            let a: Vec<&String> = categories.values().into_iter().collect::<Vec<_>>();
            a.contains(&tag)
        })
        .map(|mut tag| {
            for (cat, value) in categories {
                if value == &tag {
                    let c = match cat {
                        Category::Programming => "programming",
                        Category::WebDevelopment => "web dev",
                        Category::GameDevelopment => "game dev",
                        Category::MobileDevelopment => "mobile dev",
                    }
                    .to_string();
                    tag = c;
                }
            }
            tag
        })
        .collect()
}
