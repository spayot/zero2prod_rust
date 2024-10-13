use actix_web::{body::to_bytes, http::StatusCode, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

use super::IdempotencyKey;

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "header_pair")]
struct HeaderPairRecord {
    name: String,
    value: Vec<u8>,
}


pub async fn get_saved_response(user_id: Uuid, idempotency_key: &IdempotencyKey, pool: &PgPool) -> anyhow::Result<Option<HttpResponse>> {
    let saved_response = sqlx::query!(
        r#"
            SELECT  response_status_code,
                    response_headers as "response_headers: Vec<HeaderPairRecord>",
                    response_body
            FROM idempotency
            WHERE user_id = $1
              AND idempotency_key = $2
        "#,
        user_id,
        idempotency_key.as_ref())
    .fetch_optional(pool)
    .await
    ?;
    
    match saved_response {
        None => Ok(None),
        Some(r) => {
            let status_code = StatusCode::from_u16(r.response_status_code.try_into()?)?;
            let mut response = HttpResponse::build(status_code);
            for HeaderPairRecord {name, value} in r.response_headers {
                response.append_header((name, value));
            };

            Ok(Some(response.body(r.response_body)))
        }
    }
}


pub async fn save_response(
    user_id: Uuid, 
    idempotency_key: &IdempotencyKey, 
    pool: &PgPool, 
    http_response: HttpResponse) -> anyhow::Result<HttpResponse> {
        let (response_head, body) = http_response.into_parts();
        let status_code = response_head.status().as_u16() as i16;
        let header_pairs = {
            let mut h = Vec::with_capacity(response_head.headers().len());
            for (name, value) in response_head.headers().iter() {
                let name = name.as_str().to_owned();
                let value = value.as_bytes().to_owned();
                h.push(HeaderPairRecord {name, value});
            }
            h
        };
        let body = to_bytes(body).await.map_err(|e| anyhow::anyhow!("{}", e))?;

        sqlx::query_unchecked!(r#"
            INSERT INTO idempotency (
                user_id,
                idempotency_key,
                response_status_code,
                response_headers,
                response_body,
                created_at
            )
            VALUES ($1, $2, $3, $4, $5, now())
            "#,
            user_id,
            idempotency_key.as_ref(),
            status_code,
            header_pairs,
            body.as_ref(),
        ).execute(pool)
        .await?
        ;

        let http_response = response_head.set_body(body).map_into_boxed_body();
        Ok(http_response)
}
