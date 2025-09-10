use crate::lib::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use crate::lib::email_client::EmailClient;
use crate::lib::startup::ApplicationBaseUrl;
use actix_web::{HttpResponse, web};
use chrono::Utc;
use sqlx::PgPool;

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;
    fn try_from(form: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(&form.name)?;
        let email = SubscriberEmail::parse(&form.email)?;
        Ok(NewSubscriber { email, name })
    }
}

#[tracing::instrument(name = "Adding a new subscriber",
	skip(form, pool, email_client, base_url),
	fields(
		subscriber_email = %form.email,
		subscriber_name = %form.name))]
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
) -> HttpResponse {
    // Parse subscriber.
    let new_subscriber = match form.0.try_into() {
        Ok(form) => form,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    if insert_subscriber(&new_subscriber, &pool).await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }
    if send_confirmation_email(email_client, new_subscriber, &base_url.0)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber, base_url)
)]
pub async fn send_confirmation_email(
    email_client: web::Data<EmailClient>,
    new_subscriber: NewSubscriber,
    base_url: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?{}",
        base_url, "subscription_token=mytoken"
    );
    let html_body = format!(
        r#"Welcome to our newsletter!<br />
		Click <a href="{}">here</a> to confirm your subscriptions"#,
        confirmation_link
    );
    let plain_body = format!(
        r#"Welcome to our newsletter!
		Visit {} to confirm your subscriptions"#,
        confirmation_link
    );

    email_client
        .send_email(new_subscriber.email, "Welcome!", &html_body, &plain_body)
        .await
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(
    new_subscriber: &NewSubscriber,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        uuid::Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now(),
        "pending_confirmation"
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
