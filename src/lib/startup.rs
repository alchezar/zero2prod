use crate::greet;
use crate::lib::routes::{health_check, subscribe};
use actix_web::dev::Server;
use tracing_actix_web::TracingLogger;
use actix_web::{App, HttpServer, web};
use sqlx::PgPool;
use std::net::TcpListener;

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/", web::get().to(greet))
            .route("/{name}", web::get().to(greet))
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(web::Data::clone(&db_pool))
    })
    .listen(listener)?
    .workers(4)
    .run();

    Ok(server)
}
