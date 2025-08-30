use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero2prod::lib::configurations::{DatabaseSettings, get_configuration};
use zero2prod::lib::startup::Application;
use zero2prod::lib::startup::get_connection_pool;
use zero2prod::lib::telemetry::{get_subscriber, init_subscriber};

// Ensure that the `tracing` stack is only initialised once using `once_cell`.
static TRACING: Lazy<()> = Lazy::new(|| {
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber("zero2prod".into(), "debug".into(), std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber("zero2prod".into(), "debug".into(), std::io::sink);
        init_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

pub async fn spawn_app() -> TestApp {
    // The first time `initialize` is invoked the code in `Tracing` is executed.
    // Al other invocations will
    Lazy::force(&TRACING);

    // Randomise configuration to ensure test isolation.
    let configuration = {
        let mut conf = get_configuration().expect("Failed to read configuration.");
        // Use a different database for each test case.
        conf.database.database_name = Uuid::new_v4().to_string();
        // Use random OS port.
        conf.application.port = 0;
        conf
    };
    // Create and migrate the database.
    configure_database(&configuration.database).await;
    // Launch the application as a background task.
    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application");
    // Get the port before spawning the application.
    let address = format!("http://localhost:{}", application.port());
    let _ = tokio::spawn(application.run_until_stopped());

    TestApp {
        address,
        db_pool: get_connection_pool(&configuration),
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database.
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres.");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database");

    // Migrate database.
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
