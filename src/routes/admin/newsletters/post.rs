use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use anyhow::Context;
use sqlx::PgPool;


use crate::authentication::UserId;
use crate::utils::{e400, e500, see_other};
use crate::email_client::EmailClient;
use crate::domain::SubscriberEmail;
use crate::idempotency::{save_response, get_saved_response, IdempotencyKey};

#[derive(serde::Deserialize)]
pub struct FormData {
    title: String,
    content_html: String,
    content_text: String,
    idempotency_key: String,
}

#[tracing::instrument(
    name="Publish a newsletter to confirmed subscribers.",
    skip(form, pool, email_client, user_id)
    fields(user_id=%&*user_id)
)]
pub async fn publish_newsletter(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = user_id.into_inner();
    let FormData {title, content_html, content_text, idempotency_key} = form.0;
    let idempotency_key: IdempotencyKey = idempotency_key.try_into().map_err(e400)?;
    if let Some(saved_response) = get_saved_response(*user_id, &idempotency_key, &pool).await.map_err(e500)? {
        FlashMessage::info(format!("Your newsletter '{}' has been published.", title)).send();
        dbg!("went through flow?");
        return Ok(saved_response)
    }
    let subscribers = get_confirmed_subscribers(&pool).await.map_err(e500)?;
    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => {
                email_client.send_email(
                        &subscriber.email,
                        &title,
                        &content_html, 
                        &content_text)
                        .await
                        .with_context(|| {format!("Failed to send newsletter to {}", &subscriber.email)})
                        .map_err(e500)?;
            },
            Err(error) => {
                tracing::warn!(
                    error.cause_chain = ?error,
                    "Skipping a confirmed subscriber. \
                    Their stored contact is invalid."
                )
            }    
        };
    }
    FlashMessage::info(format!("Your newsletter '{}' has been published.", title)).send();
    let response = see_other("/admin/newsletters");
    let response = save_response(*user_id, &idempotency_key, &pool, response).await.map_err(e500)?;
    Ok(response)
}


struct ConfirmedSubscribers {
    email: SubscriberEmail,
}


#[tracing::instrument(
    name="Retrieve confirmed subscribers.",
)]
async fn get_confirmed_subscribers(pool: &PgPool) -> Result<Vec<Result<ConfirmedSubscribers, anyhow::Error>>, anyhow::Error> {

    let confirmed_subscribers = sqlx::query!(
        r#"
        SELECT email 
        FROM subscriptions 
        WHERE status = 'confirmed'"#)
    .fetch_all(pool)
    .await?
    .into_iter()
        .map(|r| match SubscriberEmail::parse(r.email) {
            Ok(email) => Ok(ConfirmedSubscribers {email}),
            Err(error) => Err(anyhow::anyhow!(error)),
        })
        .collect();
    Ok(confirmed_subscribers)

}