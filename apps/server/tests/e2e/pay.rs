use crate::helpers;
use crate::helpers::create_test_server_client_no_redirect;

#[tokio::test(flavor = "multi_thread")]
async fn pay_check() {
    let app = helpers::spawn_app().await;
    let response = create_test_server_client_no_redirect()
        .get(&app.test_server_url("/pay?amount=123&action=subscribe&currency=USD&language=en"))
        .send()
        .await
        .expect("Failed to execute the request");

    assert!(
        response.status().is_redirection(),
        "Response should be a redirection"
    );
}
