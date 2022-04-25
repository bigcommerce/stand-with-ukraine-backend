use crate::helpers::spawn_app;
use serde_json::json;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

#[tokio::test]
async fn install_request_from_bigcommerce_fails_without_query_parameters() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/bigcommerce/install", &app.address))
        .query(&[("context", "test")])
        .send()
        .await
        .expect("Failed to execute the request");

    assert!(!response.status().is_success());
    assert!(
        response.text().await.unwrap().contains("missing field"),
        "Response should complain about missing field"
    );

    let response = client
        .get(&format!("{}/bigcommerce/install", &app.address))
        .query(&[("code", "test")])
        .send()
        .await
        .expect("Failed to execute the request");

    assert!(!response.status().is_success());
    assert!(
        response.text().await.unwrap().contains("missing field"),
        "Response should complain about missing field"
    );

    let response = client
        .get(&format!("{}/bigcommerce/install", &app.address))
        .query(&[("scope", "test")])
        .send()
        .await
        .expect("Failed to execute the request");

    assert!(!response.status().is_success());
    assert!(
        response.text().await.unwrap().contains("missing field"),
        "Response should complain about missing field"
    );
}

#[tokio::test]
async fn install_request_from_bigcommerce_succeeds() {
    let app = spawn_app().await;

    Mock::given(method("POST"))
        .and(path("/oauth2/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "access_token": "ACCESS_TOKEN",
                "scope": "store_v2_orders",
                "user": {
                    "id": 24654,
                    "email": "merchant@mybigcommerce.com"
                },
                "context": "stores/STORE_HASH"
        })))
        .named("BigCommerce oauth token request")
        .expect(1)
        .mount(&app.bigcommerce_server)
        .await;

    let client = reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let response = client
        .get(&format!("{}/bigcommerce/install", &app.address))
        .query(&[
            ("context", "test-context"),
            ("scope", "test-scope"),
            ("code", "test-code"),
        ])
        .send()
        .await
        .expect("Failed to execute the request");

    assert_eq!(response.status().as_u16(), 302);
    assert!(
        response
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap()
            .contains(&app.base_url),
        "Header location should be set"
    );
}
