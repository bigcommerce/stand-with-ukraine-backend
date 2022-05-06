use swu_app::{
    bigcommerce::BCStoreInformationResponse,
    data::{StoreStatus, WidgetConfiguration},
};

use crate::{
    helpers::spawn_app,
    mocks::{
        create_script_mock, delete_script_mock, get_scripts_mock, get_store_information_mock,
        update_script_mock,
    },
};

#[tokio::test]
async fn widget_publish_request_fails_without_token_or_with_invalid_token() {
    let app = spawn_app().await;

    let configuration = WidgetConfiguration {
        style: "blue".to_string(),
        placement: "top-left".to_string(),
        charity_selections: vec!["razom".to_string()],
        modal_title: "Title!".to_string(),
        modal_body: "Body!".to_string(),
    };

    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/api/v1/configuration", &app.address))
        .json(&configuration)
        .send()
        .await
        .unwrap();

    assert!(response.status().is_client_error());

    let response = client
        .post(&format!("{}/api/v1/configuration", &app.address))
        .bearer_auth("test-token")
        .json(&configuration)
        .send()
        .await
        .unwrap();

    assert!(response.status().is_client_error());
}

#[tokio::test]
async fn widget_publish_request_succeeds() {
    let app = spawn_app().await;

    app.insert_test_store().await;

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

        let response = client
            .post(&format!("{}/api/v1/publish", &app.address))
            .bearer_auth(app.generate_local_jwt_token())
            .send()
            .await
            .expect("Failed to execute the request");

        assert!(response.status().is_success());
    }

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

        let response = client
            .post(&format!("{}/api/v1/publish", &app.address))
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

    let client = reqwest::Client::new();
    client
        .post(&format!("{}/api/v1/configuration", &app.address))
        .bearer_auth(app.generate_local_jwt_token())
        .send()
        .await
        .expect("Failed to execute the request");

    let response = client
        .post(&format!("{}/api/v1/publish", &app.address))
        .bearer_auth(app.generate_local_jwt_token())
        .send()
        .await
        .expect("Failed to execute the request");

    assert!(response.status().is_server_error());
}

#[tokio::test]
async fn widget_preview_request_succeeds() {
    let app = spawn_app().await;

    app.insert_test_store().await;

    get_store_information_mock()
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

    let client = reqwest::Client::new();
    let response = client
        .delete(&format!("{}/api/v1/publish", &app.address))
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

    assert_eq!(response.published, false);
}
