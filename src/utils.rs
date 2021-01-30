use std::collections::HashMap;

use rocket::response::{self, Responder, Response};
use rocket::{http::Status, request::Request};
use rocket_contrib::json::Json;
use serde::Serialize;

use crate::{category::Category, tags::get_twitch_tag_names, twitch_stream::TwitchStream};

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

pub fn filter_by_category(streams: Vec<TwitchStream>, category_tag: &String, categories: &HashMap<Category, String>) -> Vec<TwitchStream> {
    streams
        .into_iter()
        .filter(|stream| {
            match &stream.tag_ids {
                Some(tags) => tags.iter().any(|id| id.eq(category_tag)),
                None => false
            }
        })
    .map(|mut s| {
        s.tag_ids = Some(get_twitch_tag_names(s.tag_ids.unwrap(), categories));
        s
    })
    .collect()
}


pub fn filter_all_programming_streams<'a>(streams: Vec<TwitchStream>, tag_ids: &HashMap<Category, String>) -> Vec<TwitchStream> {
    let tag_id_vals: Vec<&String> = tag_ids.values().collect();
    streams
        .into_iter()
        .filter(|stream| {
            match &stream.tag_ids {
                Some(tags) => tags.iter().any(|id| tag_id_vals.contains(&id)),
                None => false
            }
        })
    .map(|mut s| {
        s.tag_ids = Some(get_twitch_tag_names(s.tag_ids.unwrap(), tag_ids));
        s
    })
    .collect()
}

