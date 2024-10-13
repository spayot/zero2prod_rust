use crate::helpers::{assert_is_redirect_to, spawn_app};

#[tokio::test]
async fn an_error_flash_message_is_set_on_failure() {
    // Arrange
    let test_app = spawn_app().await;

    let login_body = serde_json::json!(
        {
            "username": "invalid_username", 
            "password": "invalid_password",
        }
    );
    // Act - Part 1
    let response = test_app.post_login(&login_body).await;

    // Assert - Part 1
    assert_is_redirect_to(&response, "/login");

    // Act - Part 2
    let html = test_app.get_login_html().await;
    
    // Assert - Part 2
    assert!(html.contains("<p><i>Authentication failed</i></p>"));

    // Act - Part 3
    let html = test_app.get_login_html().await;

    // Assert - Part 3
    assert!(!html.contains("<p><i>Authentication failed</i></p>"));
    
}

#[tokio::test]
async fn redirect_to_admin_dashboard_after_login_success() {
    // Arrange
    let test_app = spawn_app().await;

    let login_body = serde_json::json!(
        {
            "username": &test_app.test_user.username, 
            "password": &test_app.test_user.password,
        }
    );
    // Act - Part 1
    let response = test_app.post_login(&login_body).await;

    // Assert - Part 1
    assert_is_redirect_to(&response, "/admin/dashboard");

    // Act - Part 2
    let html = test_app.get_admin_dashboard_html().await;
    
    // Assert - Part 2
    assert!(html.contains(&format!("Welcome, {}", test_app.test_user.username)));    
}