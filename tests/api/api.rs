use serde_json::json;
use swu_app::data::{StoreStatus, WidgetConfiguration};
use wiremock::{
    matchers::{header, method, path},
    Mock, ResponseTemplate,
};

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

#[tokio::test]
async fn save_and_read_widget_configuration() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

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

    let response = client
        .post(&format!("{}/api/v1/configuration", &app.address))
        .bearer_auth(app.generate_local_jwt_token())
        .json(&configuration)
        .send()
        .await
        .expect("Failed to execute the request");

    assert!(response.status().is_success(), "Response should be success");
    assert_eq!(
        Some(0),
        response.content_length(),
        "Content length should be 0"
    );

    let row = sqlx::query!(
        r#"
        SELECT widget_configuration FROM stores
        WHERE store_hash = $1
        "#,
        "test-store"
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();

    let widget_configuration: WidgetConfiguration =
        serde_json::value::from_value(row.widget_configuration).unwrap();

    assert_eq!(widget_configuration.charity_selections.len(), 1);
    assert_eq!(widget_configuration.style, "blue");
    assert_eq!(widget_configuration.placement, "top-left");
    assert_eq!(widget_configuration.modal_body, "Body!");
    assert_eq!(widget_configuration.modal_title, "Title!");

    let response_widget_configuration = client
        .get(&format!("{}/api/v1/configuration", &app.address))
        .bearer_auth(app.generate_local_jwt_token())
        .send()
        .await
        .expect("Failed to execute the request")
        .json::<WidgetConfiguration>()
        .await
        .unwrap();

    assert_eq!(response_widget_configuration.charity_selections.len(), 1);
    assert_eq!(response_widget_configuration.style, "blue");
    assert_eq!(response_widget_configuration.placement, "top-left");
    assert_eq!(response_widget_configuration.modal_body, "Body!");
    assert_eq!(response_widget_configuration.modal_title, "Title!");
}

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

    Mock::given(method("POST"))
        .and(path("/stores/test-store/v3/content/scripts"))
        .and(header("X-Auth-Token", "test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "uuid": "095be615-a8ad-4c33-8e9c-c7612fbf6c9f",
                "date_created": "2019-08-24T14:15:22Z",
                "date_modified": "2019-08-24T14:15:22Z",
                "description": "string",
                "html": "string",
                "src": "https://code.jquery.com/jquery-3.2.1.min.js",
                "auto_uninstall": true,
                "load_method": "default",
                "location": "head",
                "visibility": "storefront",
                "kind": "src",
                "api_client_id": "string",
                "consent_category": "essential",
                "enabled": true,
                "channel_id": 1
            },
            "meta": {}
        })))
        .named("BigCommerce oauth token request")
        .expect(1)
        .mount(&app.bigcommerce_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/stores/test-store/v3/content/scripts"))
        .and(header("X-Auth-Token", "test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": [
                  {
                    "uuid": "095be615-a8ad-4c33-8e9c-c7612fbf6c9f",
                    "date_created": "2019-08-24T14:15:22Z",
                    "date_modified": "2019-08-24T14:15:22Z",
                    "description": "string",
                    "html": "string",
                    "src": "https://code.jquery.com/jquery-3.2.1.min.js",
                    "auto_uninstall": true,
                    "load_method": "default",
                    "location": "head",
                    "visibility": "storefront",
                    "kind": "src",
                    "api_client_id": "string",
                    "consent_category": "essential",
                    "enabled": true,
                    "channel_id": 1
                  }
                ],
                "meta": {
                  "pagination": {
                    "total": 36,
                    "count": 36,
                    "per_page": 50,
                    "current_page": 1,
                    "total_pages": 1,
                    "links": {
                      "previous": "string",
                      "current": "?page=1&limit=50",
                      "next": "string"
                    }
                  }
                }
        })))
        .named("BigCommerce oauth token request")
        .expect(1)
        .mount(&app.bigcommerce_server)
        .await;

    Mock::given(method("DELETE"))
        .and(path(
            "/stores/test-store/v3/content/scripts/095be615-a8ad-4c33-8e9c-c7612fbf6c9f",
        ))
        .and(header("X-Auth-Token", "test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": 204,
            "title": "string",
            "type": "string",
            "instance": "string"
        })))
        .named("BigCommerce oauth token request")
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
