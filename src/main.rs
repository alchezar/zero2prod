// IKinder

//! main.rs

use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::configurations::get_configuration;
use zero2prod::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Panic if we can't read configuration.
    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    // We have removed the hard-coded `8000` - it's now coming from our settings.
    let address = format!("localhost:{}", configuration.application_port);

    // Bubble up the io::Error if we failed to bind the address
    // Otherwise call .await on our Server.
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await
}
