use std::fmt::Write;

use actix_web::{cookie::Cookie, http::header::ContentType, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;


pub async fn login_form(flash_messages: IncomingFlashMessages) -> HttpResponse {
    let mut error_html = String::new();
    for m in flash_messages.iter() {
        writeln!(error_html, "<p><i>{}</i></p>", m.content()).unwrap();
    }
    let mut response = HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(include_str!("login_form.html"), error_html = error_html));

    response
        .add_removal_cookie(&Cookie::new("_flash", ""))
        .unwrap();
    response
}