use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;

use crate::{authentication::UserId, session_state::TypedSession, utils::see_other};

pub async fn log_out(_user_id: web::ReqData<UserId>, session: TypedSession) -> Result<HttpResponse, actix_web::Error> {
    
    session.log_out();
    FlashMessage::info("You have successfully logged out.").send(); 
    Ok(see_other("/login"))
}