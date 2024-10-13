use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;
use unicode_segmentation::UnicodeSegmentation;

use crate::{authentication::{validate_credentials, AuthError, Credentials, UserId}, routes::dashboard::get_username, utils::see_other};
use crate::utils::e500;


#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>,
}

#[tracing::instrument(
    name="Change user password",
    skip(form, pool),
)]
pub async fn change_password(
    form: web::Form<FormData>, 
    pool: web::Data<PgPool>,
    user_id: web::ReqData<UserId>
) -> Result<HttpResponse, actix_web::Error> {
    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        FlashMessage::error("You entered two different new passwords - the field values must match.").send();
        return Ok(see_other("/admin/password"))
    }
    let user_id = user_id.into_inner();
    let username = get_username(*user_id, &pool).await.map_err(e500)?;
    let credentials = Credentials {
        username,
        password: form.0.current_password,
    };
    if let Err(e) = validate_credentials(credentials, &pool).await {
        return match e {
            AuthError::InvalidCredentials(_) => {
                FlashMessage::error("The current password is incorrect.").send();
                Ok(see_other("/admin/password"))
            },
            AuthError::UnexpectedError(e) => Err(e500(e).into())
        }
    };

    let password_length = form.0.new_password.expose_secret().graphemes(true).count();
    
    if (password_length < 12) || (password_length > 129) {
        FlashMessage::error("The new password should be between 12 and 129 characters.").send();
        return Ok(see_other("/admin/password"))
    }
    
    crate::authentication::change_password(*user_id, form.0.new_password, &pool).await.map_err(e500)?;
    FlashMessage::error("Your password has been changed.").send();
    Ok(see_other("/admin/password"))
}

