use swu_app::{
    bigcommerce::BCStoreInformationResponse,
    data::{StoreStatus, WidgetConfiguration},
};
use wiremock::{
    matchers::{header, method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::spawn_app;

#[tokio::test]
async fn widget_publish_request_succeeds() {
    let app = spawn_app().await;

    sqlx::query!(
        r#"
        INSERT INTO stores (id, store_hash, access_token, installed_at, uninstalled) 
        VALUES (gen_random_uuid(), 'test-store', 'test-token', '2021-04-20 00:00:00-07'::timestamptz, false)
        "#,
    )
    .execute(&app.db_pool)
    .await
    .unwrap();

    let configuration = WidgetConfiguration {
        style: "blue".to_string(),
        placement: "top-left".to_string(),
        charity_selections: vec!["razom".to_string()],
        modal_title: "Title!".to_string(),
        modal_body: "Body!".to_string(),
    };

    let client = reqwest::Client::new();
    client
        .post(&format!("{}/api/v1/configuration", &app.address))
        .bearer_auth(app.generate_local_jwt_token())
        .json(&configuration)
        .send()
        .await
        .expect("Failed to execute the request");

    let response = client
        .get(&format!("{}/api/v1/publish", &app.address))
        .bearer_auth(app.generate_local_jwt_token())
        .send()
        .await
        .expect("Failed to execute the request")
        .json::<StoreStatus>()
        .await
        .expect("Invalid response format");

    assert_eq!(response.published, false);

    let create_scripts_response: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/create_script.json"))
            .expect("Failed to parse file");

    Mock::given(method("POST"))
        .and(path("/stores/test-store/v3/content/scripts"))
        .and(header("X-Auth-Token", "test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_scripts_response))
        .named("BigCommerce create script request")
        .expect(1)
        .mount(&app.bigcommerce_server)
        .await;

    let get_scripts_response: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/get_scripts.json"))
            .expect("Failed to parse file");

    Mock::given(method("GET"))
        .and(path("/stores/test-store/v3/content/scripts"))
        .and(header("X-Auth-Token", "test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(get_scripts_response))
        .named("BigCommerce get scripts request")
        .expect(1)
        .mount(&app.bigcommerce_server)
        .await;

    let delete_script_response: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/delete_script.json"))
            .expect("Failed to parse file");

    Mock::given(method("DELETE"))
        .and(path(
            "/stores/test-store/v3/content/scripts/095be615-a8ad-4c33-8e9c-c7612fbf6c9f",
        ))
        .and(header("X-Auth-Token", "test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(delete_script_response))
        .named("BigCommerce delete script request")
        .expect(1)
        .mount(&app.bigcommerce_server)
        .await;

    let response = client
        .post(&format!("{}/api/v1/publish", &app.address))
        .bearer_auth(app.generate_local_jwt_token())
        .send()
        .await
        .expect("Failed to execute the request");

    assert!(response.status().is_success());

    let response = client
        .get(&format!("{}/api/v1/publish", &app.address))
        .bearer_auth(app.generate_local_jwt_token())
        .send()
        .await
        .expect("Failed to execute the request")
        .json::<StoreStatus>()
        .await
        .expect("Invalid response format");

    assert_eq!(response.published, true);
}

#[tokio::test]
async fn widget_preview_request_succeeds() {
    let app = spawn_app().await;

    sqlx::query!(
        r#"
        INSERT INTO stores (id, store_hash, access_token, installed_at, uninstalled) 
        VALUES (gen_random_uuid(), 'test-store', 'test-token', '2021-04-20 00:00:00-07'::timestamptz, false)
        "#,
    )
    .execute(&app.db_pool)
    .await
    .unwrap();

    let store_information_response: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/get_store.json"))
            .expect("Failed to parse file");

    Mock::given(method("GET"))
        .and(path("/stores/test-store/v2/store"))
        .and(header("X-Auth-Token", "test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(store_information_response))
        .named("BigCommerce get store information")
        .expect(1)
        .mount(&app.bigcommerce_server)
        .await;

    let client = reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let response = client
        .get(&format!("{}/api/v1/preview", &app.address))
        .bearer_auth(app.generate_local_jwt_token())
        .send()
        .await
        .expect("Failed to execute the request")
        .json::<BCStoreInformationResponse>()
        .await
        .expect("Failed to deserialize response");

    assert_eq!(
        response.secure_url,
        "https://test-store-t85.mybigcommerce.com"
    );
}
