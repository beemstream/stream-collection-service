use std::io::Cursor;

use miniserde::{json, Serialize};
use rocket::http::ContentType;
use rocket::response::{self, Responder, Response};
use rocket::{http::Status, request::Request};

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
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let string_json = json::to_string(&self.data);
        Response::build()
            .header(ContentType::JSON)
            .status(self.status_code)
            .sized_body(string_json.len(), Cursor::new(string_json))
            .ok()
    }
}
