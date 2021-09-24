use rocket::{catch, request::Request};

#[catch(404)]
pub fn not_found(_: &Request) {}

#[catch(401)]
pub fn unauthorized(_: &Request) {}
