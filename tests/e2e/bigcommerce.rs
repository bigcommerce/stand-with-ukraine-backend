use crate::{helpers::spawn_app, mocks::get_oauth2_token_mock};
use secrecy::Secret;
use swu_app::{
    bigcommerce::{BCStore, BCUser},
    data::write_store_credentials,
};

#[tokio::test]
async fn install_request_fails_without_bigcommerce_response() {
    let app = spawn_app().await;

    let client = reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let response = client
        .get(&format!("{}/bigcommerce/install", &app.address))
        .query(&[
            ("context", "stores/STORE_HASH"),
            ("scope", "test-scope"),
            ("code", "test-code"),
        ])
        .send()
        .await
        .expect("Failed to execute the request");

    assert!(response.status().is_server_error());
}

#[tokio::test]
async fn install_request_fails_without_query_parameters() {
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
async fn install_request_succeeds() {
    let app = spawn_app().await;

    get_oauth2_token_mock()
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
            ("context", "stores/STORE_HASH"),
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

    let row = sqlx::query!(
        r#"
        SELECT store_hash, access_token, uninstalled, published FROM stores
        WHERE store_hash = 'STORE_HASH'
        "#
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();

    assert_eq!(row.access_token, "ACCESS_TOKEN");
    assert_eq!(row.store_hash, "STORE_HASH");
}

#[tokio::test]
async fn load_request_fails_with_bad_token() {
    let app = spawn_app().await;

    let client = reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let response = client
        .get(&format!("{}/bigcommerce/load", &app.address))
        .query(&[("signed_payload_jwt", "bad-token")])
        .send()
        .await
        .expect("Failed to execute the request");

    assert!(response.status().is_client_error());

    let user = BCUser {
        id: 1,
        email: "user@test.com".to_string(),
    };

    let response = client
        .get(&format!("{}/bigcommerce/load", &app.address))
        .query(&[(
            "signed_payload_jwt",
            app.generate_bc_jwt_token_with_params("bad-hash", &user, &user),
        )])
        .send()
        .await
        .expect("Failed to execute the request");

    assert!(response.status().is_server_error());
}

#[tokio::test]
async fn load_request_succeeds() {
    let app = spawn_app().await;

    let client = reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let response = client
        .get(&format!("{}/bigcommerce/load", &app.address))
        .query(&[("signed_payload_jwt", &app.generate_bc_jwt_token())])
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

#[tokio::test]
async fn uninstall_request_succeeds() {
    let app = spawn_app().await;

    let client = reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let store = BCStore {
        store_hash: "test-store".to_string(),
        access_token: Secret::from("test-token".to_string()),
    };
    write_store_credentials(&store, &app.db_pool)
        .await
        .expect("Failed to initialize store");

    let response = client
        .get(&format!("{}/bigcommerce/uninstall", &app.address))
        .query(&[("signed_payload_jwt", &app.generate_bc_jwt_token())])
        .send()
        .await
        .expect("Failed to execute the request");

    let row = sqlx::query!(
        r#"
        SELECT uninstalled FROM stores
        WHERE store_hash = 'test-store'
        "#
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();

    assert_eq!(row.uninstalled, true);
    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn uninstall_request_fails_with_non_owner() {
    let app = spawn_app().await;

    let client = reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let store = BCStore {
        store_hash: "test-store".to_string(),
        access_token: Secret::from("test-token".to_string()),
    };
    write_store_credentials(&store, &app.db_pool)
        .await
        .expect("Failed to initialize store");

    let owner = BCUser {
        id: 1,
        email: "owner@test.com".to_string(),
    };
    let user = BCUser {
        id: 2,
        email: "user@test.com".to_string(),
    };

    let response = client
        .get(&format!("{}/bigcommerce/uninstall", &app.address))
        .query(&[(
            "signed_payload_jwt",
            &app.generate_bc_jwt_token_with_params("test-store", &owner, &user),
        )])
        .send()
        .await
        .expect("Failed to execute the request");

    let row = sqlx::query!(
        r#"
        SELECT uninstalled FROM stores
        WHERE store_hash = 'test-store'
        "#
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();

    assert_eq!(row.uninstalled, false);
    assert!(response.status().is_client_error());
}
