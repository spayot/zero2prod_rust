
use crate::helpers::{assert_is_redirect_to, spawn_app};


#[tokio::test]
async fn you_must_be_logged_in_to_access_the_admin_dashboard() {
    // Arrange
    let test_app = spawn_app().await;

    // Act 
    let response = test_app.get_admin_dashboard().await;

    // Assert
    assert_is_redirect_to(&response, "/login");    
}


#[tokio::test]
async fn logout_clears_session_state() {
    // Arrange
    let test_app = spawn_app().await;

    // Act - Part 1 - Login
    let response = test_app.valid_login().await; 
    assert_is_redirect_to(&response, "/admin/dashboard");

    // Act - Part 2 - Follow the redirect
    let html_page = test_app.get_admin_dashboard_html().await;
    assert!(html_page.contains(&format!("Welcome, {}", test_app.test_user.username)));

    // Act - Part 3 - Logout
    let response = test_app.post_logout().await;
    assert_is_redirect_to(&response, "/login");
    

    // Act - Part 4 - Follow the redirect
    let html_page = test_app.get_login_html().await;
    assert!(html_page.contains("You have successfully logged out."));

    // Act - Part 5 - Attempt to load admin panel
    let response = test_app.get_admin_dashboard().await;
    assert_is_redirect_to(&response, "/login");

    // Assert
}
