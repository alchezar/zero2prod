// IKinder

//! lib.rs

pub mod configurations;
pub mod routes;
pub mod startup;

use actix_web::{HttpRequest, Responder};

async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {name}!")
}
