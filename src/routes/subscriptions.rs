﻿// IKinder

use actix_web::{HttpResponse, web};
use chrono::Utc;
use sqlx::PgPool;

#[derive(serde::Deserialize)]
pub struct FromData {
    name: String,
    email: String,
}

pub async fn subscribe(form: web::Form<FromData>, pool: web::Data<PgPool>) -> HttpResponse {
    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        uuid::Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(pool.get_ref())
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            println!("Failed to execute query {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
