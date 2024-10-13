use anyhow::Context;
use actix_web::{http::StatusCode, web, HttpResponse, ResponseError};
use sqlx::PgPool;
use uuid::Uuid;

use crate::routes::error_chain_fmt;

#[tracing::instrument(
    name = "Confirm a pending subscriber",
    skip(params)
)]
pub async fn confirm(
    params: web::Query<Parameters>, pool: web::Data<PgPool>
)-> Result<HttpResponse, SubscriptionConfirmError> {
    let id = get_subscriber_id(&params.subscription_token, &pool)
        .await
        .context("Failed to retrieve subscriber id associated with the confirmation token.")?
        .ok_or(SubscriptionConfirmError::UnknownToken)?;
    

    confirm_subscription(&pool, id)
        .await
        .context("Failed to update database as confirmed.")?;
    Ok(HttpResponse::Ok().finish())
}

#[derive(thiserror::Error)]
pub enum SubscriptionConfirmError {

    #[error("There is no subscriber associated with the provided token.")]
    UnknownToken,
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error)
}

impl std::fmt::Debug for SubscriptionConfirmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
     }
}

impl ResponseError for SubscriptionConfirmError {
    fn status_code(&self) -> StatusCode {
        match self {
            SubscriptionConfirmError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            SubscriptionConfirmError::UnknownToken => StatusCode::UNAUTHORIZED,
        }
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(
    name = "get subscriber id from the database given a token",
    skip(pool, subscription_token)
)]
pub async fn get_subscriber_id(
    subscription_token: &str, pool: &PgPool
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1"#,
        subscription_token
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(result
        .map(|r| r.subscriber_id))
}

#[tracing::instrument(
    name = "Confirm subscription after click on email link",
    skip(pool),
)]
pub async fn confirm_subscription(pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE subscriptions SET status = 'confirmed' WHERE id = $1
        "#,
        subscriber_id,
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to confirm subscriber: {:?}", e);
        e
    })?;
    Ok(())
}
