use crate::helpers::spawn_app;

#[tokio::test(flavor = "multi_thread")]
async fn insert_event_without_store_does_not_create_record() {
    let app = spawn_app().await;

    assert_eq!(app.get_widget_events("test-store").await.count(), 0);

    let response = app
        .test_client
        .post(app.test_server_url("/api/v2/widget-event"))
        .query(&[("event", "widget-opened"), ("store_hash", "test-store")])
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 200);

    assert_eq!(app.get_widget_events("test-store").await.count(), 0);

    assert_eq!(
        app.get_charity_visited_events("test-store").await.count(),
        0
    );

    let response = app
        .test_client
        .post(app.test_server_url("/api/v2/charity-event"))
        .query(&[
            ("charity", "razom"),
            ("store_hash", "test-store"),
            ("event", "support-clicked"),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 200);

    assert_eq!(
        app.get_charity_visited_events("test-store").await.count(),
        0
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn insert_event_after_store_created_creates_record() {
    let app = spawn_app().await;
    app.insert_test_store().await;

    assert_eq!(app.get_widget_events("test-store").await.count(), 0);

    let response = app
        .test_client
        .post(app.test_server_url("/api/v2/widget-event"))
        .query(&[("event", "widget-opened"), ("store_hash", "test-store")])
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 200);

    assert_eq!(
        app.get_widget_events("test-store")
            .await
            .filter(|event| event == "widget-opened")
            .count(),
        1
    );

    assert_eq!(
        app.get_charity_visited_events("test-store").await.count(),
        0
    );

    let response = app
        .test_client
        .post(app.test_server_url("/api/v2/charity-event"))
        .query(&[
            ("charity", "razom"),
            ("store_hash", "test-store"),
            ("event", "support-clicked"),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 200);

    assert_eq!(
        app.get_charity_visited_events("test-store")
            .await
            .filter(|(charity, event_type)| charity == "razom" && event_type == "support-clicked")
            .count(),
        1
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn insert_event_using_universal_creates_record() {
    let app = spawn_app().await;
    app.insert_test_store().await;

    assert_eq!(app.get_universal_widget_events().await.count(), 0);

    let response = app
        .test_client
        .post(app.test_server_url("/api/v2/widget-event"))
        .query(&[("event", "widget-opened"), ("store_hash", "universal")])
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 200);

    assert_eq!(
        app.get_universal_widget_events()
            .await
            .filter(|event| event == "widget-opened")
            .count(),
        1
    );

    assert_eq!(app.get_universal_charity_visited_events().await.count(), 0);

    let response = app
        .test_client
        .post(app.test_server_url("/api/v2/charity-event"))
        .query(&[
            ("charity", "razom"),
            ("store_hash", "universal"),
            ("event", "support-clicked"),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 200);

    assert_eq!(
        app.get_universal_charity_visited_events()
            .await
            .filter(|(charity, event_type)| charity == "razom" && event_type == "support-clicked")
            .count(),
        1
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn submit_general_feedback_without_required_or_invalid_fields_does_not_create_record() {
    let app = spawn_app().await;

    assert_eq!(app.get_form_feedback_submissions().await.count(), 0);

    // invalid email
    let response = app
        .test_client
        .post(app.test_server_url("/api/v2/feedback-form"))
        .query(&[("name", "Test"), ("message", "Test"), ("email", "test")])
        .send()
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 400);
    assert!(
        response.text().await.unwrap().contains("invalid value"),
        "Response should complain about invalid email field"
    );

    assert_eq!(app.get_form_feedback_submissions().await.count(), 0);

    // empty required field
    let response = app
        .test_client
        .post(app.test_server_url("/api/v2/feedback-form"))
        .query(&[("name", "Test"), ("message", "Test")])
        .send()
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 400);
    assert!(
        response.text().await.unwrap().contains("missing field"),
        "Response should complain about missing email field"
    );

    assert_eq!(app.get_form_feedback_submissions().await.count(), 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn submit_general_feedback_should_create_record() {
    let app = spawn_app().await;

    assert_eq!(app.get_form_feedback_submissions().await.count(), 0);

    // valid
    let response = app
        .test_client
        .post(app.test_server_url("/api/v2/feedback-form"))
        .query(&[
            ("name", "Test"),
            ("message", "Test"),
            ("email", "test@test.com"),
        ])
        .send()
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(app.get_form_feedback_submissions().await.count(), 1);
}

#[tokio::test(flavor = "multi_thread")]
async fn submit_universal_configurator_event_should_create_record() {
    let app = spawn_app().await;

    assert_eq!(
        app.get_universal_configurator_submissions().await.count(),
        0
    );

    // valid
    let response = app
        .test_client
        .post(app.test_server_url("/api/v2/universal-event"))
        .query(&[("event_type", "generate-code")])
        .send()
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(
        app.get_universal_configurator_submissions().await.count(),
        1
    );
}
