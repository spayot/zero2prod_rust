use actix_web::{http::header::ContentType, web, HttpResponse};
use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{authentication::UserId, utils::e500};

pub async fn admin_dashboard(user_id: web::ReqData<UserId>, pool: web::Data<PgPool>) -> Result<HttpResponse, actix_web::Error> {
    let username = get_username(*user_id.into_inner(), &pool).await.map_err(e500)?;
            
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(include_str!("dashboard.html"), username = username)
        )
    )
}

#[tracing::instrument(
    name="Get username.",
    skip(pool)
)]
pub async fn get_username(user_id: Uuid, pool: &PgPool) -> anyhow::Result<String> {
    let result = sqlx::query!(
        r#"SELECT username FROM users WHERE user_id = $1"#,
        user_id
    )
    .fetch_one(pool)
    .await
    .context("Failed to execute query to retrieve a username.")?;
    
    Ok(result.username)
}


