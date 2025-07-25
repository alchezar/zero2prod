// IKinder

//! main.rs

use zero2prod::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Bubble up the io::Error if we failed to bind the address
    // Otherwise call .await on our Server.
    let listener = std::net::TcpListener::bind("localhost:8000")?;
    run(listener)?.await
}
