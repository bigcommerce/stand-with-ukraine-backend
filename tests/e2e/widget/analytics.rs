use crate::helpers::spawn_app;

#[tokio::test]
async fn insert_event_without_store_does_not_create_record() {
    let app = spawn_app().await;

    assert_eq!(app.get_widget_events("test-store").await.count(), 0);

    let response = app
        .test_client
        .post(app.test_server_url("/api/v2/widget-event"))
        .query(&[("event", "opened"), ("store_hash", "test-store")])
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
        .query(&[("charity", "razom"), ("store_hash", "test-store")])
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 200);

    assert_eq!(
        app.get_charity_visited_events("test-store").await.count(),
        0
    );
}

#[tokio::test]
async fn insert_event_after_store_created_creates_record() {
    let app = spawn_app().await;
    app.insert_test_store().await;

    assert_eq!(app.get_widget_events("test-store").await.count(), 0);

    let response = app
        .test_client
        .post(app.test_server_url("/api/v2/widget-event"))
        .query(&[("event", "opened"), ("store_hash", "test-store")])
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 200);

    assert_eq!(
        app.get_widget_events("test-store")
            .await
            .filter(|event| event == r#""opened""#)
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
        .query(&[("charity", "razom"), ("store_hash", "test-store")])
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 200);

    assert_eq!(
        app.get_charity_visited_events("test-store")
            .await
            .filter(|charity| charity == r#""razom""#)
            .count(),
        1
    );
}
