extern crate google_sheets4 as sheets4;
use sheets4::api::{BatchUpdateValuesRequest, ValueRange};
use swu_app::startup::get_connection_pool;

use crate::{
    configuration::Configuration,
    sheets::{create_bulk_updates_for_sheet, get_sheets_client},
};

pub async fn run(configuration: Configuration) {
    let sheets = get_sheets_client(
        configuration.sheets.credential_path.as_str(),
        configuration.sheets.token_cache_path.as_str(),
    )
    .await;

    let db_pool = get_connection_pool(&configuration.database);

    let stores = sqlx::query!("SELECT * FROM stores")
        .fetch_all(&db_pool)
        .await
        .unwrap();

    let feedback_list = sqlx::query!("SELECT * FROM unpublish_events")
        .fetch_all(&db_pool)
        .await
        .unwrap();

    let charity_events = sqlx::query!(
        r#"
        SELECT charity, event_type, count(*)
        FROM charity_events
        GROUP BY event_type, charity
        ORDER BY charity, event_type
        "#
    )
    .fetch_all(&db_pool)
    .await
    .unwrap();

    let widget_events = sqlx::query!(
        r#"
        SELECT event_type, count(*)
        FROM widget_events
        GROUP BY event_type
        "#
    )
    .fetch_all(&db_pool)
    .await
    .unwrap();

    let spreadsheet_id = configuration.sheets.spreadsheet_id.as_str();
    let mut updates: Vec<ValueRange> = create_bulk_updates_for_sheet(
        &sheets,
        spreadsheet_id,
        "stores",
        stores
            .iter()
            .map(|store| {
                vec![
                    store.id.to_string(),
                    store.store_hash.to_owned(),
                    store.installed_at.to_string(),
                    store.published.to_string(),
                    store.uninstalled.to_string(),
                ]
            })
            .collect(),
    )
    .await;

    updates.extend(
        create_bulk_updates_for_sheet(
            &sheets,
            spreadsheet_id,
            "feedback",
            feedback_list
                .iter()
                .map(|feedback| {
                    vec![
                        feedback.id.to_string(),
                        feedback.store_hash.to_owned(),
                        feedback.unpublished_at.to_string(),
                        feedback.reason.to_owned(),
                    ]
                })
                .collect(),
        )
        .await,
    );

    updates.extend(
        create_bulk_updates_for_sheet(
            &sheets,
            spreadsheet_id,
            "charity-events",
            charity_events
                .iter()
                .map(|charity_event| {
                    vec![
                        format!("{}:{}", charity_event.charity, charity_event.event_type),
                        charity_event.charity.to_owned(),
                        charity_event.event_type.to_owned(),
                        charity_event.count.unwrap().to_string(),
                    ]
                })
                .collect(),
        )
        .await,
    );

    updates.extend(
        create_bulk_updates_for_sheet(
            &sheets,
            spreadsheet_id,
            "widget-events",
            widget_events
                .iter()
                .map(|widget_event| {
                    vec![
                        widget_event.event_type.to_owned(),
                        widget_event.count.unwrap().to_string(),
                    ]
                })
                .collect(),
        )
        .await,
    );

    let request = BatchUpdateValuesRequest {
        data: Some(updates),
        value_input_option: Some("USER_ENTERED".to_owned()),
        response_date_time_render_option: None,
        response_value_render_option: None,
        include_values_in_response: Some(false),
    };

    println!("{:?}", &request);

    let response = sheets
        .spreadsheets()
        .values_batch_update(request, spreadsheet_id)
        .doit()
        .await
        .unwrap();

    println!("{:?}", response);
}
