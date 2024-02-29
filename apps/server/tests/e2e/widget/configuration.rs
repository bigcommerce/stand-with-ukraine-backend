use swu_app::data::WidgetConfiguration;

use crate::helpers::{get_widget_configuration, spawn_app};

#[tokio::test(flavor = "multi_thread")]
async fn save_widget_configuration_fails_with_invalid_config() {
    let app = spawn_app().await;

    let bad_configuration = serde_json::json!({
        "bad": "value"
    });

    let response = app
        .test_client
        .post(&app.test_server_url("/api/v1/configuration"))
        .bearer_auth(app.generate_local_jwt_token())
        .json(&bad_configuration)
        .send()
        .await
        .expect("Failed to execute the request");

    assert!(response.status().is_client_error());
}

#[tokio::test(flavor = "multi_thread")]
async fn save_widget_configuration_fails_when_store_not_defined() {
    let app = spawn_app().await;

    let response = app
        .test_client
        .post(&app.test_server_url("/api/v1/configuration"))
        .bearer_auth(app.generate_local_jwt_token())
        .json(&get_widget_configuration())
        .send()
        .await
        .expect("Failed to execute the request");

    assert!(response.status().is_server_error());
}

#[tokio::test(flavor = "multi_thread")]
async fn read_widget_configuration_fails_with_no_store() {
    let app = spawn_app().await;

    let response = app
        .test_client
        .get(&app.test_server_url("/api/v1/configuration"))
        .bearer_auth(app.generate_local_jwt_token())
        .send()
        .await
        .expect("Failed to execute the request");

    assert!(response.status().is_server_error());
}

#[tokio::test(flavor = "multi_thread")]
async fn save_and_read_widget_configuration() {
    let app = spawn_app().await;

    app.insert_test_store().await;

    let response = app
        .test_client
        .post(&app.test_server_url("/api/v1/configuration"))
        .bearer_auth(app.generate_local_jwt_token())
        .json(&get_widget_configuration())
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

    let response_widget_configuration = app
        .test_client
        .get(&app.test_server_url("/api/v1/configuration"))
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
