use crate::helpers::spawn_app;

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health_check", &app.address))
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
