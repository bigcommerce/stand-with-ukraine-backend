use swu_app::data::WidgetConfiguration;

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
