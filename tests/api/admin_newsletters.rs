use wiremock::{matchers::{method, path}, Mock, ResponseTemplate};

use crate::helpers::{assert_is_redirect_to, spawn_app, ConfirmationLinks, TestApp};

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    // Arrange
    let test_app = spawn_app().await;
    test_app.valid_login().await;

    create_unconfirmed_subscriber(&test_app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&test_app.email_server)
        .await;

    
    // Act
    let response = test_app.post_newsletter(&newsletter_body()).await;

    // Assert
    assert_is_redirect_to(&response, "/admin/newsletters");    
}


#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    // Arrange
    let test_app = spawn_app().await;
    test_app.valid_login().await;

    create_confirmed_subscriber(&test_app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

     // Act
     let response = test_app.post_newsletter(&newsletter_body()).await;

     // Assert
     assert_is_redirect_to(&response, "/admin/newsletters");    

    // Act 2 - validate success message has been injected
     let html_page = test_app.get_send_newsletters_html().await;
     assert!(html_page.contains("has been published.</i></p>"))
}

#[tokio::test]
async fn newsletters_returns_400_for_invalid_data() {
    let test_app = spawn_app().await;
    test_app.valid_login().await;

    let test_cases = [
        (
            serde_json::json!({
                "content_html": "<p> HTML Content </p>",
                "content_text": "Text Content",
            }), 
            "missing title"
        ),
        (
            serde_json::json!({"title": "Title"}), 
            "missing content"
        ),
    ];

    for (newsletter_body, msg) in test_cases {
        let response = test_app.post_newsletter(&newsletter_body).await;
        assert_eq!(
            response.status().as_u16(), 
            400,
            "the newsletter API did not fail with 400 when body has {}.",
            msg
        )
    }
    
}

#[tokio::test]
async fn you_must_be_logged_in_to_post_newsletters() {
    // Arrange
    let test_app = spawn_app().await;

    // Act
    let response = test_app.post_newsletter(&newsletter_body()).await;
    
    // Assert
    assert_is_redirect_to(&response, "/login");    
}



#[tokio::test]
async fn newsletter_creation_is_idempotent() {
    // Arrange
    let test_app = spawn_app().await;
    test_app.valid_login().await;

    create_confirmed_subscriber(&test_app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

     // Act - Part 1 - publish newsletter
     let body = newsletter_body();
     let response = test_app.post_newsletter(&body).await;
     assert_is_redirect_to(&response, "/admin/newsletters");    

    // Act - Part 2 - validate success message has been injected
     let html_page = test_app.get_send_newsletters_html().await;
     assert!(html_page.contains("has been published.</i></p>"));

     // Act - Part 3 - publish newsletter **again**
     let response = test_app.post_newsletter(&body).await;
     assert_is_redirect_to(&response, "/admin/newsletters");    

    // Act - Part 4 - validate success message has been injected
     let html_page = test_app.get_send_newsletters_html().await;
     assert!(html_page.contains("has been published.</i></p>"));


     // Assert: mock verifies on Drop that only **1** call was made to endpoint, not 2
}

async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
    let body = "name=john%20doe&email=john_doe%40mail.com"; 
    
    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;
    
    app.post_subscriptions(body.into())
        .await
        .error_for_status()
        .unwrap();
    
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    app.get_confirmation_links(email_request)
}

async fn create_confirmed_subscriber(app: &TestApp) {
    // create unconfirmed subscriber
    let confirmation_links = create_unconfirmed_subscriber(app).await;

    // confirm subscriber
    reqwest::Client::new()
        .get(confirmation_links.html)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}



fn newsletter_body() -> serde_json::Value {
    serde_json::json!({
        "title": "Title",
        "content_html": "<p> HTML Content </p>",
        "content_text": "Text Content",
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
    })
}