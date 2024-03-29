use std::collections::HashMap;

use rocket::info;
use rocket::{http::Status, request::Request};
use rocket::{
    response::{self, Responder, Response},
    serde::json::Json,
};
use serde::Serialize;

use crate::category::Category;
use crate::clients::twitch::TwitchStream;

pub struct JsonResponse<T> {
    data: T,
    status_code: Status,
}

impl<T> JsonResponse<T>
where
    T: Serialize,
{
    pub fn new(data: T, status_code: Status) -> Self {
        Self { data, status_code }
    }
}

#[rocket::async_trait]
impl<'r, T: Serialize> Responder<'r, 'static> for JsonResponse<T> {
    fn respond_to(self, request: &'r Request<'_>) -> response::Result<'static> {
        Response::build_from(Json(self.data).respond_to(request).unwrap())
            .status(self.status_code)
            .ok()
    }
}

const TITLE_BLACKLIST: &[&str] = &["minecraft", "fortnite", "pokemon"];

pub fn filter_by_category(
    streams: Vec<TwitchStream>,
    category_tag: &str,
    all_tags: &HashMap<String, String>,
) -> Vec<TwitchStream> {
    streams
        .into_iter()
        .filter(|stream| {
            // let is_matched_tag = match &stream.tag_ids {
            //     Some(tags) => tags.iter().any(|id| id.eq(category_tag)),
            //     None => false,
            // };

            let is_blacklist = TITLE_BLACKLIST
                .iter()
                .any(|blacklist| stream.title.to_lowercase().contains(blacklist));

            !is_blacklist
        })
        .map(|mut s| {
            s.tag_ids = {
                if s.tag_ids.is_none() {
                    Some(vec!["programming".to_owned()])
                } else {
                    Some(
                        s.tag_ids
                            .unwrap()
                            .into_iter()
                            .map(|ids| all_tags.get(&ids).unwrap().to_owned())
                            .collect(),
                    )
                }
            };
            s
        })
        .collect()
}

pub fn filter_all_programming_streams(
    streams: Vec<TwitchStream>,
    tag_ids: &HashMap<Category, String>,
    all_tags: &HashMap<String, String>,
) -> Vec<TwitchStream> {
    let tag_id_vals: Vec<&String> = tag_ids.values().collect();
    streams
        .into_iter()
        .filter(|stream| {
            // let is_matched_tag = match &stream.tag_ids {
            //     Some(tags) => tags.iter().any(|id| tag_id_vals.contains(&id)),
            //     None => false,
            // };
            let is_blacklist = TITLE_BLACKLIST
                .iter()
                .any(|blacklist| stream.title.to_lowercase().contains(blacklist));

            (stream.game_id == "1469308723") && !is_blacklist
        })
        .map(|mut s| {
            info!("{:#?}", s.tag_ids);
            s.tag_ids = {
                if s.tag_ids.is_none() {
                    Some(vec!["programming".to_owned()])
                } else {
                    Some(
                        s.tag_ids
                            .unwrap()
                            .into_iter()
                            // .filter(|ids| all_tags.get(ids).is_some())
                            .map(|ids| all_tags.get(&ids).unwrap().to_owned())
                            .collect(),
                    )
                }
            };
            s
        })
        .collect()
}
