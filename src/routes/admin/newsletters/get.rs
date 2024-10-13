use actix_web::{http::header::ContentType, web, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;
use std::fmt::Write;

use crate::authentication::UserId;


pub async fn send_newsletter_form(_user_id: web::ReqData<UserId>, flash_messages: IncomingFlashMessages) -> Result<HttpResponse, actix_web::Error> {
    let mut msg_html = String::new();
    for m in flash_messages.iter() {
        writeln!(msg_html, "<p><i>{}</i></p>", m.content()).unwrap();
    }
    let idempotency_key = uuid::Uuid::new_v4();
    let response = HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(include_str!("newsletter_form.html"), msg_html = msg_html, idempotency_key= idempotency_key));

    Ok(response)
}