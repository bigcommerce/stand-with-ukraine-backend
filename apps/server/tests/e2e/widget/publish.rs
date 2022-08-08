use swu_app::{bigcommerce::store::BCStoreInformationResponse, data::StoreStatus};

use crate::{
    helpers::{create_test_server_client_no_redirect, get_widget_configuration, spawn_app},
    mocks::{
        create_script_mock, delete_script_mock, get_scripts_mock, get_store_information_mock,
        update_script_mock,
    },
};

#[tokio::test]
async fn widget_publish_request_fails_without_token_or_with_invalid_token() {
    let app = spawn_app().await;

    let response = app
        .test_client
        .post(&app.test_server_url("/api/v1/configuration"))
        .json(&get_widget_configuration())
        .send()
        .await
        .unwrap();

    assert!(response.status().is_client_error());

    let response = app
        .test_client
        .post(&app.test_server_url("/api/v1/configuration"))
        .bearer_auth("test-token")
        .json(&get_widget_configuration())
        .send()
        .await
        .unwrap();

    assert!(response.status().is_client_error());
}

#[tokio::test]
async fn widget_publish_request_succeeds() {
    let app = spawn_app().await;

    app.insert_test_store().await;

    app.test_client
        .post(&app.test_server_url("/api/v1/configuration"))
        .bearer_auth(app.generate_local_jwt_token())
        .json(&get_widget_configuration())
        .send()
        .await
        .expect("Failed to execute the request");

    let response = app
        .test_client
        .get(&app.test_server_url("/api/v1/publish"))
        .bearer_auth(app.generate_local_jwt_token())
        .send()
        .await
        .expect("Failed to execute the request")
        .json::<StoreStatus>()
        .await
        .expect("Invalid response format");

    assert!(!response.published);

    // first publish - should use create request to bc
    {
        let _get_guard = get_scripts_mock(false)
            .expect(1)
            .mount_as_scoped(&app.bigcommerce_server)
            .await;

        let _create_guard = create_script_mock()
            .expect(1)
            .mount_as_scoped(&app.bigcommerce_server)
            .await;

        let response = app
            .test_client
            .post(&app.test_server_url("/api/v1/publish"))
            .bearer_auth(app.generate_local_jwt_token())
            .send()
            .await
            .expect("Failed to execute the request");

        assert!(response.status().is_success());
    }

    let response = app
        .test_client
        .get(&app.test_server_url("/api/v1/publish"))
        .bearer_auth(app.generate_local_jwt_token())
        .send()
        .await
        .expect("Failed to execute the request")
        .json::<StoreStatus>()
        .await
        .expect("Invalid response format");

    assert!(response.published);

    // second publish - should use put request to bc to update existing script
    {
        let _get_guard = get_scripts_mock(true)
            .expect(1)
            .mount_as_scoped(&app.bigcommerce_server)
            .await;

        let _update_guard = update_script_mock()
            .expect(1)
            .mount_as_scoped(&app.bigcommerce_server)
            .await;

        let response = app
            .test_client
            .post(&app.test_server_url("/api/v1/publish"))
            .bearer_auth(app.generate_local_jwt_token())
            .send()
            .await
            .expect("Failed to execute the request");

        assert!(response.status().is_success());
    }
}

#[tokio::test]
async fn widget_publish_request_fails_without_configuration_saved() {
    let app = spawn_app().await;

    app.insert_test_store().await;

    app.test_client
        .post(&app.test_server_url("/api/v1/configuration"))
        .bearer_auth(app.generate_local_jwt_token())
        .send()
        .await
        .expect("Failed to execute the request");

    let response = app
        .test_client
        .post(&app.test_server_url("/api/v1/publish"))
        .bearer_auth(app.generate_local_jwt_token())
        .send()
        .await
        .expect("Failed to execute the request");

    assert!(response.status().is_server_error());
}

#[tokio::test]
async fn widget_publish_request_fails_without_bigcommerce_server_response() {
    let app = spawn_app().await;

    app.insert_test_store().await;

    app.test_client
        .post(&app.test_server_url("/api/v1/configuration"))
        .bearer_auth(app.generate_local_jwt_token())
        .json(&get_widget_configuration())
        .send()
        .await
        .expect("Failed to execute the request");

    let response = app
        .test_client
        .post(&app.test_server_url("/api/v1/publish"))
        .bearer_auth(app.generate_local_jwt_token())
        .send()
        .await
        .expect("Failed to execute the request");

    assert!(response.status().is_server_error());
}

#[tokio::test]
async fn get_published_status_fails_without_store() {
    let app = spawn_app().await;

    let response = app
        .test_client
        .get(&app.test_server_url("/api/v1/publish"))
        .bearer_auth(app.generate_local_jwt_token())
        .send()
        .await
        .expect("Failed to execute the request");

    assert!(response.status().is_server_error());
}

#[tokio::test]
async fn widget_preview_request_fails_without_store() {
    let app = spawn_app().await;

    let response = app
        .test_client
        .get(&app.test_server_url("/api/v1/preview"))
        .bearer_auth(app.generate_local_jwt_token())
        .send()
        .await
        .expect("Failed to execute the request");

    assert!(response.status().is_server_error());
}

#[tokio::test]
async fn widget_preview_request_succeeds() {
    let app = spawn_app().await;
    let client = create_test_server_client_no_redirect();

    app.insert_test_store().await;

    get_store_information_mock()
        .expect(1)
        .mount(&app.bigcommerce_server)
        .await;

    let response = client
        .get(&app.test_server_url("/api/v1/preview"))
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

#[tokio::test]
async fn widget_remove_request_fails_without_store() {
    let app = spawn_app().await;

    let response = app
        .test_client
        .delete(&app.test_server_url("/api/v1/publish"))
        .bearer_auth(app.generate_local_jwt_token())
        .send()
        .await
        .expect("Failed to execute the request");

    assert!(response.status().is_server_error());
}

#[tokio::test]
async fn widget_remove_request_succeeds() {
    let app = spawn_app().await;

    app.insert_test_store().await;

    get_scripts_mock(false)
        .expect(1)
        .mount(&app.bigcommerce_server)
        .await;

    delete_script_mock()
        .expect(1)
        .mount(&app.bigcommerce_server)
        .await;

    let response = app
        .test_client
        .delete(&app.test_server_url("/api/v1/publish"))
        .bearer_auth(app.generate_local_jwt_token())
        .send()
        .await
        .expect("Failed to execute the request");

    assert!(response.status().is_success());

    let response = app
        .test_client
        .get(&app.test_server_url("/api/v1/publish"))
        .bearer_auth(app.generate_local_jwt_token())
        .send()
        .await
        .expect("Failed to execute the request")
        .json::<StoreStatus>()
        .await
        .expect("Invalid response format");

    assert!(!response.published);
}

#[tokio::test]
async fn widget_remove_request_with_feedback_succeeds() {
    let app = spawn_app().await;

    app.insert_test_store().await;

    get_scripts_mock(false)
        .expect(1)
        .mount(&app.bigcommerce_server)
        .await;

    delete_script_mock()
        .expect(1)
        .mount(&app.bigcommerce_server)
        .await;

    let response = app
        .test_client
        .delete(&app.test_server_url("/api/v1/publish"))
        .query(&[("reason", "I did not like the design!")])
        .bearer_auth(app.generate_local_jwt_token())
        .send()
        .await
        .expect("Failed to execute the request");

    assert!(response.status().is_success());

    let response = app
        .test_client
        .get(&app.test_server_url("/api/v1/publish"))
        .bearer_auth(app.generate_local_jwt_token())
        .send()
        .await
        .expect("Failed to execute the request")
        .json::<StoreStatus>()
        .await
        .expect("Invalid response format");

    assert!(!response.published);

    let rows = sqlx::query!(
        "SELECT reason FROM unpublish_events WHERE store_hash = $1",
        "test-store"
    )
    .fetch_all(&app.db_pool)
    .await
    .unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].reason, "I did not like the design!");
}
