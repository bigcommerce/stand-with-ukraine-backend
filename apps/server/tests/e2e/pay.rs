use crate::helpers;

#[tokio::test]
async fn pay_check() {
    let app = helpers::spawn_app().await;
    let response = app
        .test_client
        .get(&app.test_server_url("/pay?sum=123&action=subscribe&currency=usd&language=en"))
        .send()
        .await
        .expect("Failed to execute the request");
    assert!(
        response.status().is_redirection(),
        "Response should be a redirection"
    );
}
