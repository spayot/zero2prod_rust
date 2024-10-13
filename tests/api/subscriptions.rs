use wiremock::{Mock, ResponseTemplate};
use wiremock::matchers::{path, method};

use crate::helpers::spawn_app;

#[tokio::test]
async fn subsccribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let test_app = spawn_app().await;
    
    // Act
    let body = "name=john%20doe&email=john_doe%40mail.com";
    
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    let response = test_app.post_subscriptions(body.into()).await;

    // Assert
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_persists_the_new_subscriber() {
    // Arrange
    let test_app = spawn_app().await;
    
    // Act
    let body = "name=john%20doe&email=john_doe%40mail.com";
    
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    test_app.post_subscriptions(body.into()).await;

    // Assert
    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "john_doe@mail.com");
    assert_eq!(saved.name, "john doe");
    assert_eq!(saved.status, "pending_confirmation")
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let test_app = spawn_app().await;
    let test_cases = vec![
        ("name=silevin%20poyat", "missing email"),
        ("email=sylvain%40example.com", "missing name"),
        ("", "missing name and email")
    ];

    // Act
    for (body, message) in test_cases {
        let response = test_app.post_subscriptions(body.into()).await;
        assert_eq!(400, response.status().as_u16(), "The API did not fail with 400 Bad Request when payload was {message}.");
    }

}

#[tokio::test]
async fn subscribe_returns_a_400_when_fields_are_invalid() {
    let test_app = spawn_app().await;
    let test_cases = vec![
        ("name=%20&email=john%40example.com", "empty name"),
        ("name=john&email=", "empty email"),
        ("name=john&email=johnexample.com", "missing @"),
        // ("name=john&email=john@example", "missing top-level domain"),
    ];
    
    for (body, message) in test_cases {
        let response = test_app.post_subscriptions(body.into()).await;
        assert_eq!(400, response.status().as_u16(), "The API did not fail with 400 Bad Request when payload was {message}.");
    }
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    // Arrange
    let test_app = spawn_app().await;
    let body = "name=john%20doe&email=john_doe%40mail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    // Act
    test_app.post_subscriptions(body.into()).await;

    // Assert
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
    // Arrange
    let test_app = spawn_app().await;
    let body = "name=john%20doe&email=john_doe%40mail.com"; 

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

   // Act
    test_app.post_subscriptions(body.into()).await; 

    // Assert
    let email_request = &test_app.email_server.received_requests().await.unwrap()[0];
    let confirmation_link = test_app.get_confirmation_links(email_request);
    assert_eq!(confirmation_link.html, confirmation_link.plain_text);
}

#[tokio::test]
async fn subscribe_fails_if_there_is_a_fatal_database_error() {
    // Arrange
    let test_app = spawn_app().await;
    let body = "name=john%20doe&email=john_doe%40mail.com"; 
    
    sqlx::query!("ALTER table subscriptions DROP COLUMN email;",)
        .execute(&test_app.db_pool)
        .await
        .unwrap();
    
    // Act
    let response = test_app.post_subscriptions(body.into()).await;
    
    // Assert
    assert_eq!(response.status().as_u16(), 500);
}