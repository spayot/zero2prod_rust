use actix_web::{error::InternalError, http::{header::LOCATION, StatusCode}, web, HttpResponse, ResponseError};
use actix_web_flash_messages::FlashMessage;
use secrecy::Secret;
use sqlx::PgPool;

use crate::{authentication::{validate_credentials, AuthError, Credentials}, routes::error_chain_fmt, session_state::TypedSession};

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    password: Secret<String>,
}

#[tracing::instrument(
    name="Login a user",
    skip(form, pool, session),
    fields(
        username = tracing::field::Empty,
        user_id = tracing::field::Empty
    )
)]
pub async fn login(
    form: web::Form<FormData>, 
    pool: web::Data<PgPool>,
    session: TypedSession,
) -> Result<HttpResponse, InternalError<LoginError>> {
    let credentials = Credentials {
        username: form.0.username,
        password: form.0.password,
    };
    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));
    match validate_credentials(credentials, &pool).await {
        Ok(user_id) => {
            session.renew();
            session 
                .insert_user_id(user_id)
                .map_err(|e| login_redirect(LoginError::UnexpectedError(e.into())))?;
            tracing::Span::current()
                .record("user_id", &tracing::field::display(&user_id));
            Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/admin/dashboard"))
                .finish())
        },
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
                AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
            };
            
            Err(login_redirect(e))
        }
    }
}


#[derive(thiserror::Error)]
pub enum LoginError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
}

impl std::fmt::Debug for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
     }
}

impl ResponseError for LoginError {
    fn status_code(&self) -> StatusCode {
       StatusCode::SEE_OTHER
    }
}

fn login_redirect(error: LoginError) -> InternalError<LoginError> {
    FlashMessage::error(error.to_string()).send();

    let response = HttpResponse::SeeOther()
        .insert_header((LOCATION, format!("/login")))
        .finish();
    InternalError::from_response(error, response)
}