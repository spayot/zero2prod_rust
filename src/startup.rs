use std::net::TcpListener;

use actix_session::storage::RedisSessionStore;
use actix_web::cookie::Key;
use actix_web::{web, App, HttpServer};
use actix_web::dev::Server;
use actix_web_flash_messages::storage::CookieMessageStore;
use actix_web_flash_messages::FlashMessagesFramework;
use actix_session::SessionMiddleware;
use actix_web::middleware::from_fn;
use secrecy::{ExposeSecret, Secret};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

use crate::authentication::reject_anonymous_users;
use crate::email_client::EmailClient;
use crate::routes::{admin_dashboard, change_password, change_password_form, confirm, health_check, home, log_out, login, login_form, publish_newsletter, send_newsletter_form, subscribe};
use crate::configuration::{DatabaseSettings, Settings};

pub struct ApplicationBaseUrl(pub String);

#[derive(Clone)]
pub struct HmacSecret(pub Secret<String>);

async fn run(
    listener: TcpListener, 
    db_pool: PgPool, 
    email_client: EmailClient, 
    base_url: String,
    hmac_secret: Secret<String>,
    redis_uri: Secret<String>,

) -> Result<Server, anyhow::Error> {
    let secret_key = Key::from(hmac_secret.expose_secret().as_bytes());
    let storage_backend = CookieMessageStore::builder(secret_key.clone()).build();
    let message_framework = FlashMessagesFramework::builder(storage_backend).build();
    let redis_store = RedisSessionStore::new(redis_uri.expose_secret()).await?;
    let db_pool = web::Data::new(db_pool);
    let email_client = web::Data::new(email_client);
    let base_url = web::Data::new(ApplicationBaseUrl(base_url));
    let hmac_secret = web::Data::new(HmacSecret(hmac_secret));
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(message_framework.clone())
            .wrap(SessionMiddleware::new(redis_store.clone(), secret_key.clone()))
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm))
            .route("/", web::get().to(home))
            .route("/login", web::get().to(login_form))
            .route("/login", web::post().to(login))
            .service(
                web::scope("/admin")
                .wrap(from_fn(reject_anonymous_users))
                .route("/dashboard", web::get().to(admin_dashboard))
                .route("/password", web::get().to(change_password_form))
                .route("/password", web::post().to(change_password))
                .route("/logout", web::post().to(log_out))
                .route("/newsletters", web::post().to(publish_newsletter))
                .route("/newsletters", web::get().to(send_newsletter_form))
            )

            .app_data(db_pool.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
            .app_data(hmac_secret.clone())
    })
    .listen(listener)?
    .run();
    
    Ok(server)
}



pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .connect_lazy_with(configuration.with_db())
}

pub struct Application {
    server: Server,
    port: u16,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, anyhow::Error> {
        let connection_pool = get_connection_pool(&configuration.database);

        let sender_email = configuration.email_client.sender().expect("invalid sender email address.");
        let timeout = configuration.email_client.timeout();
        let email_client = EmailClient::new(
            configuration.email_client.base_url, 
            sender_email, 
            configuration.email_client.authorization_token, 
            timeout);

        let address = format!("{}:{}", configuration.application.host, configuration.application.port);
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();
        
        let server = run(listener,
            connection_pool, 
            email_client, 
            configuration.application.base_url,
            configuration.application.hmac_secret,
            configuration.redis_uri,
            ).await?;
        Ok(Self { server, port })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}


