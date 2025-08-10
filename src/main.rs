// IKinder

//! main.rs

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use zero2prod::lib::configurations::get_configuration;
use zero2prod::lib::startup::run;
use zero2prod::lib::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // Panic if we can't read configuration.
    let configuration = get_configuration().expect("Failed to read configuration.");
    // let connection_pool = PgPool::connect_lazy(&configuration.database.connection_string())
    // 	.expect("Failed to create Postgres connection pool.");
    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy(&configuration.database.connection_string())
        .expect("Failed to create Postgres connection pool.");
    // We have removed the hard-coded `8000` - it's now coming from our settings.
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );

    // Bubble up the io::Error if we failed to bind the address
    // Otherwise call .await on our Server.
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await
}
