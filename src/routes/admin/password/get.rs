use actix_web::{http::header::ContentType, web, HttpResponse};
use actix_web_flash_messages::{IncomingFlashMessages, Level};
use std::fmt::Write;

use crate::authentication::UserId;


pub async fn change_password_form(_user_id: web::ReqData<UserId>, flash_messages: IncomingFlashMessages) -> Result<HttpResponse, actix_web::Error> {
    let mut msg_html = String::new();
    for m in flash_messages.iter().filter(|m| m.level() == Level::Error) {
        writeln!(msg_html, "<p><i>{}</i></p>", m.content()).unwrap();
    }

    let response = HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(include_str!("password_form.html"), msg_html = msg_html));

    Ok(response)
}