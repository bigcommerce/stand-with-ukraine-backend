use swu_app::authentication::create_jwt;

use crate::helpers::spawn_app;

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let token = create_jwt(&"test", app.jwt_secret).unwrap();

    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/api/v1/health_check", &app.address))
        .bearer_auth(token)
        .send()
        .await
        .expect("Failed to execute the request");

    assert!(response.status().is_success(), "Response should be success");
    assert_eq!(
        Some(0),
        response.content_length(),
        "Content length should be 0"
    );
}

#[tokio::test]
async fn api_token_is_required() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/api/v1/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute the request");

    assert_eq!(
        response.status().as_u16(),
        401,
        "Response should be Unauthorized Status"
    );
}
