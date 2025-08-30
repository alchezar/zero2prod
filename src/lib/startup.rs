use crate::greet;
use crate::lib::configurations::Setting;
use crate::lib::email_client::EmailClient;
use crate::lib::routes::{health_check, subscribe};
use actix_web::dev::Server;
use actix_web::{App, HttpServer, web};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

/// A new type to hold the newly build server and its port.
pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    /// We have converted the build function into a constructor for
    /// `Application`.
    pub async fn build(configuration: Setting) -> Result<Self, std::io::Error> {
        let connection_pool = get_connection_pool(&configuration);
        let email_client = configuration
            .email_client
            .try_into()
            .expect("Invalid sender email address.");
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(&address)?;
        let port = listener.local_addr()?.port();
        let server = run(listener, connection_pool, email_client)?;

        Ok(Self { port, server })
    }
	pub fn port(&self) -> u16 {
		self.port
	}
    /// A more expressive name that makes it clear that this function only
    /// returns when the application is stopped.
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn get_connection_pool(configuration: &Setting) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_millis(
            configuration.email_client.timeout_ms,
        ))
        .connect_lazy_with(configuration.database.with_db())
}

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);
    let email_client = web::Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/", web::get().to(greet))
            .route("/{name}", web::get().to(greet))
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(web::Data::clone(&db_pool))
            .app_data(web::Data::clone(&email_client))
    })
    .listen(listener)?
    .workers(4)
    .run();

    Ok(server)
}
