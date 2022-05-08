use crate::helpers::spawn_app;

#[tokio::test]
async fn success() {
    let app = spawn_app().await;

    let response = app
        .test_client
        .get(&app.test_server_url("/health_check"))
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
