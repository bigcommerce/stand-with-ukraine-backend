pub mod bigcommerce;
pub mod widget;

pub mod helpers;
pub mod mocks;
pub mod pay;

#[tokio::test(flavor = "multi_thread")]
async fn health_check() {
    let app = helpers::spawn_app().await;

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
