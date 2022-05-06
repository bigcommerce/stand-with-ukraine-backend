use wiremock::{
    matchers::{header, method, path},
    Mock, ResponseTemplate,
};

pub fn get_scripts_mock(existing: bool) -> Mock {
    let get_scripts_response: serde_json::Value = serde_json::from_str(if existing {
        include_str!("get_scripts_existing.json")
    } else {
        include_str!("get_scripts.json")
    })
    .expect("Failed to parse file");

    Mock::given(method("GET"))
        .and(path("/stores/test-store/v3/content/scripts"))
        .and(header("X-Auth-Token", "test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&get_scripts_response))
        .named("BigCommerce get scripts request")
}

pub fn create_script_mock() -> Mock {
    let create_scripts_response: serde_json::Value =
        serde_json::from_str(include_str!("create_script.json")).expect("Failed to parse file");

    Mock::given(method("POST"))
        .and(path("/stores/test-store/v3/content/scripts"))
        .and(header("X-Auth-Token", "test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&create_scripts_response))
        .named("BigCommerce create script request")
}

pub fn update_script_mock() -> Mock {
    let update_scripts_response: serde_json::Value =
        serde_json::from_str(include_str!("create_script.json")).expect("Failed to parse file");

    Mock::given(method("PUT"))
        .and(path(
            "/stores/test-store/v3/content/scripts/095be615-a8ad-4c33-8e9c-c7612fbf6c9f",
        ))
        .and(header("X-Auth-Token", "test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(update_scripts_response))
        .named("BigCommerce update script request")
}

pub fn delete_script_mock() -> Mock {
    let delete_script_response: serde_json::Value =
        serde_json::from_str(include_str!("delete_script.json")).expect("Failed to parse file");

    Mock::given(method("DELETE"))
        .and(path(
            "/stores/test-store/v3/content/scripts/095be615-a8ad-4c33-8e9c-c7612fbf6c9f",
        ))
        .and(header("X-Auth-Token", "test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&delete_script_response))
        .named("BigCommerce delete script request")
}

pub fn get_store_information_mock() -> Mock {
    let store_information_response: serde_json::Value =
        serde_json::from_str(include_str!("get_store.json")).expect("Failed to parse file");

    Mock::given(method("GET"))
        .and(path("/stores/test-store/v2/store"))
        .and(header("X-Auth-Token", "test-token"))
        .and(header("Accept", "application/json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&store_information_response))
        .named("BigCommerce get store information")
}

pub fn get_oauth2_token_mock() -> Mock {
    let oauth2_token_response: serde_json::Value =
        serde_json::from_str(include_str!("get_oauth2_token.json")).expect("Failed to parse file");

    Mock::given(method("POST"))
        .and(path("/oauth2/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&oauth2_token_response))
        .named("BigCommerce oauth token request")
}
