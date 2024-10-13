use uuid::Uuid;

use crate::helpers::{assert_is_redirect_to, spawn_app};

#[tokio::test]
async fn you_must_be_logged_in_to_see_the_change_password_form() {
    // Arrange
    let test_app = spawn_app().await;

    // Act 
    let response = test_app.get_change_password().await;

    // Assert
    assert_is_redirect_to(&response, "/login");    
}


#[tokio::test]
async fn you_must_be_logged_in_to_change_your_password() {
    // Arrange
    let test_app = spawn_app().await;

    // Act 
    let new_password = Uuid::new_v4().to_string();
    let body = serde_json::json!(
        {
            "current_password": Uuid::new_v4().to_string(),
            "new_password": &new_password, 
            "new_password_check": &new_password,
        }
    );
    let response = test_app.post_change_password(&body).await;


    // Assert
    assert_is_redirect_to(&response, "/login");    
}

#[tokio::test]
async fn new_passwords_must_match() {
    // Arrange
    let test_app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();
    let another_new_password = Uuid::new_v4().to_string(); 

    // Act - Part 1: Login
    test_app.valid_login().await;
    

    // Act - Part 2: Try to Change Password
    let body = serde_json::json!(
        {
            "current_password": &test_app.test_user.password,
            "new_password": &new_password, 
            "new_password_check": &another_new_password,
        }
    );
    let response = test_app.post_change_password(&body).await;


    // Assert - Part 2
    assert_is_redirect_to(&response, "/admin/password");    

    // Act - Part 3: Follow the redirect
    let html_page = test_app.get_change_password_html().await;
    assert!(html_page.contains(
        "<p><i>You entered two different new passwords - \
        the field values must match.</i></p>"));
}

#[tokio::test]
async fn the_current_password_is_incorrect() {
    // Arrange
    let test_app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();
    let wrong_password = Uuid::new_v4().to_string(); 

    // Act - Part 1: Login
    test_app.valid_login().await;

    // Act - Part 2: Try to Change Password
    let body = serde_json::json!(
        {
            "current_password": &wrong_password,
            "new_password": &new_password, 
            "new_password_check": &new_password,
        }
    );
    let response = test_app.post_change_password(&body).await;


    // Assert - Part 2
    assert_is_redirect_to(&response, "/admin/password");    

    // Act - Part 3: Follow the redirect
    let html_page = test_app.get_change_password_html().await;
    assert!(html_page.contains(
        "<p><i>The current password is incorrect.</i></p>"));
}

#[tokio::test]
async fn short_passwords_are_rejected() {
    // Arrange
    let test_app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string()[..10].to_string();

    // Act - Part 1: Login
    test_app.valid_login().await;

    // Act - Part 2: Try to Change Password
    let body = serde_json::json!(
        {
            "current_password": &test_app.test_user.password,
            "new_password": &new_password, 
            "new_password_check": &new_password,
        }
    );
    let response = test_app.post_change_password(&body).await;


    // Assert - Part 2
    assert_is_redirect_to(&response, "/admin/password");    

    // Act - Part 3: Follow the redirect
    let html_page = test_app.get_change_password_html().await;
    assert!(html_page.contains(
        "<p><i>The new password should be between 12 and 129 characters.</i></p>"));
}

#[tokio::test]
async fn changing_password_works() {
    // Arrange
    let test_app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();

    // Act - Part 1 - Login
    let login_body = serde_json::json!({
        "username": &test_app.test_user.username,
        "password": &test_app.test_user.password,
    });
    let response = test_app.post_login(&login_body).await;
    assert_is_redirect_to(&response, "/admin/dashboard");

    // Act - Part 2 - Change Password
    let change_password_body = serde_json::json!(
        {
            "current_password": &test_app.test_user.password,
            "new_password": &new_password, 
            "new_password_check": &new_password,
        }
    );
     
    let response= test_app.post_change_password(&change_password_body).await;
    assert_is_redirect_to(&response, "/admin/password");

    // Act - Part 3 - Follow the redirect
    let html_page = test_app.get_change_password_html().await;
    assert!(html_page.contains("Your password has been changed"));

    // Act - Part 4 - Logout
    let response = test_app.post_logout().await;
    assert_is_redirect_to(&response, "/login");

    // Act - Part 5 - Follow the redirect
    let html_page = test_app.get_login_html().await;
    assert!(html_page.contains("You have successfully logged out."));

    // Act - Part 6 - Login with new password
    let login_body = serde_json::json!({
        "username": &test_app.test_user.username,
        "password": &new_password,
    });
    let response = test_app.post_login(&login_body).await;

    assert_is_redirect_to(&response, "/admin/dashboard");
}