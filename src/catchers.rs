use rocket::{catch, request::Request};

#[catch(404)]
pub fn not_found(_: &Request) {
    ()
}
