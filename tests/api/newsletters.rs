use crate::helpers::{assert_is_redirect_to, spawn_app, ConfirmationLinks, TestApp};
use uuid::Uuid;
use wiremock::{
    matchers::{any, method, path},
    Mock, ResponseTemplate,
};

async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
    let body = "name=le_guin&email=ursula_le_guin@gmail.com";
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

    let email_request = &app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();
    app.get_confirmation_link(&email_request)
}

async fn create_confirmed_subscriber(app: &TestApp) {
    let confirmation_link = create_unconfirmed_subscriber(app).await;
    reqwest::get(confirmation_link.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

#[tokio::test]
async fn deny_anonymous_user_to_see_publish_newsletter_form() {
    let app = spawn_app().await;
    let response = app.get_newsletters().await;
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn able_to_see_publish_newsletter_form_after_login() {
    let app = spawn_app().await;
    app.test_user.login(&app).await;

    let html_page = app.get_newsletters_html().await;
    assert!(html_page.contains("Publish Newsletter"));
}

#[tokio::test]
async fn deny_anonymous_user_to_publish_newsletter() {
    let app = spawn_app().await;
    let response = app
        .post_newsletters(&serde_json::json!({
            "title": "Newsletter Title",
            "text_content": "Newsletter body as plain text",
            "html_content": "<p>Newsletter body as html</p>",
            "idempotency_key": Uuid::new_v4().to_string(),
        }))
        .await;
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    let app = spawn_app().await;
    create_unconfirmed_subscriber(&app).await;
    app.test_user.login(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        // We assert that no request is fired at Postmark!
        .expect(0)
        .mount(&app.email_server)
        .await;

    let response = app
        .post_newsletters(&serde_json::json!({
            "title": "Newsletter Title",
            "text_content": "Newsletter body as plain text",
            "html_content": "<p>Newsletter body as html</p>",
            "idempotency_key": Uuid::new_v4().to_string(),
        }))
        .await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    let html_page = app.get_newsletters_html().await;
    assert!(html_page.contains("<p><i>The newsletter issue has been published!</i></p>"))
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.test_user.login(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let response = app
        .post_newsletters(&serde_json::json!({
            "title": "Newsletter Title",
            "text_content": "Newsletter body as plain text",
            "html_content": "<p>Newsletter body as html</p>",
            "idempotency_key": Uuid::new_v4().to_string(),
        }))
        .await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    let html_page = app.get_newsletters_html().await;
    assert!(html_page.contains("<p><i>The newsletter issue has been published!</i></p>"))
}

#[tokio::test]
async fn newsletters_creation_is_idempotent() {
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.test_user.login(&app).await;

    // Check idempotent by email_server only fire once
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter Title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as html</p>",
        "idempotency_key": Uuid::new_v4().to_string(),
    });

    // publish
    let response = app.post_newsletters(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");
    let html_page = app.get_newsletters_html().await;
    assert!(html_page.contains("<p><i>The newsletter issue has been published!</i></p>"));

    // duplicate
    let response = app.post_newsletters(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");
    let html_page = app.get_newsletters_html().await;
    assert!(html_page.contains("<p><i>The newsletter issue has been published!</i></p>"));
}
