pub mod lib {
    pub mod configurations;
    pub mod domain;
    pub mod email_client;
    pub mod routes;
    pub mod startup;
    pub mod telemetry;
}

use actix_web::{HttpRequest, Responder};

async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {name}!")
}
