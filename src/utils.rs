use std::collections::HashMap;

use rocket::response::{self, Responder, Response};
use rocket::{http::Status, request::Request};
use rocket_contrib::json::Json;
use serde::Serialize;

use crate::{category::Category, twitch_stream::TwitchStream};

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

pub fn filter_by_category(streams: Vec<TwitchStream>, category_tag: &String) -> Vec<TwitchStream> {
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
        .collect()
}

