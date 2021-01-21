use rocket::response::{self, Responder, Response};
use rocket::{http::Status, request::Request};
use rocket_contrib::json::Json;
use serde::Serialize;

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
