extern crate google_sheets4 as sheets4;
use sheets4::api::{BatchUpdateValuesRequest, ValueRange};
use sqlx::PgPool;
use swu_app::startup::get_connection_pool;
use time::OffsetDateTime;

use crate::{
    configuration::Configuration,
    sheets::{create_bulk_updates_for_sheet, get_sheets_client, Rows},
    time::{format_date, get_week_start_end},
};

pub async fn run(configuration: Configuration) {
    let sheets = get_sheets_client(
        configuration.sheets.credential_path.as_str(),
        configuration.sheets.token_cache_path.as_str(),
    )
    .await;

    let db_pool = get_connection_pool(&configuration.database);
    let (week_start, week_end) = get_week_start_end(None);

    let spreadsheet_id = configuration.sheets.spreadsheet_id.as_str();
    let mut updates: Vec<ValueRange> = create_bulk_updates_for_sheet(
        &sheets,
        spreadsheet_id,
        "stores",
        get_store_status_rows(&db_pool).await,
    )
    .await;

    updates.extend(
        create_bulk_updates_for_sheet(
            &sheets,
            spreadsheet_id,
            "uninstall-feedback",
            get_uninstall_feedback_rows(&db_pool).await,
        )
        .await,
    );

    updates.extend(
        create_bulk_updates_for_sheet(
            &sheets,
            spreadsheet_id,
            "website-feedback",
            get_website_feedback_rows(&db_pool).await,
        )
        .await,
    );

    updates.extend(
        create_bulk_updates_for_sheet(
            &sheets,
            spreadsheet_id,
            "charity-events",
            get_charity_event_summary_rows(&db_pool, week_start, week_end).await,
        )
        .await,
    );

    updates.extend(
        create_bulk_updates_for_sheet(
            &sheets,
            spreadsheet_id,
            "widget-events",
            get_widget_event_summary_rows(&db_pool, week_start, week_end).await,
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

pub async fn get_store_status_rows(db_pool: &PgPool) -> Rows {
    sqlx::query!("SELECT * FROM stores")
        .fetch_all(db_pool)
        .await
        .unwrap()
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
        .collect()
}

pub async fn get_uninstall_feedback_rows(db_pool: &PgPool) -> Rows {
    sqlx::query!("SELECT * FROM unpublish_events")
        .fetch_all(db_pool)
        .await
        .unwrap()
        .iter()
        .map(|feedback| {
            vec![
                feedback.id.to_string(),
                feedback.store_hash.to_owned(),
                feedback.unpublished_at.to_string(),
                feedback.reason.to_owned(),
            ]
        })
        .collect()
}

pub async fn get_website_feedback_rows(db_pool: &PgPool) -> Rows {
    sqlx::query!("SELECT * FROM feedback_form")
        .fetch_all(db_pool)
        .await
        .unwrap()
        .iter()
        .map(|feedback| {
            vec![
                feedback.id.to_string(),
                feedback.submitted_at.to_string(),
                feedback.name.to_owned(),
                feedback.email.to_owned(),
                feedback.message.to_owned(),
            ]
        })
        .collect()
}

pub async fn get_charity_event_summary_rows(
    db_pool: &PgPool,
    start_date: OffsetDateTime,
    end_date: OffsetDateTime,
) -> Rows {
    vec![
        vec![vec![format!(
            "⎯⎯⎯⎯⎯ {} to {} ⎯⎯⎯⎯⎯",
            format_date(start_date),
            format_date(end_date)
        )]],
        sqlx::query!(
            r#"
            SELECT charity, event_type, count(*)
            FROM charity_events
            WHERE created_at >= $1 and created_at <= $2
            GROUP BY event_type, charity
            ORDER BY event_type, charity
            "#,
            start_date,
            end_date
        )
        .fetch_all(db_pool)
        .await
        .unwrap()
        .iter()
        .map(|charity_event| {
            vec![
                format!(
                    "{}:{} {} to {}",
                    charity_event.charity,
                    charity_event.event_type,
                    format_date(start_date),
                    format_date(end_date),
                ),
                charity_event.charity.to_owned(),
                charity_event.event_type.to_owned(),
                charity_event.count.unwrap().to_string(),
            ]
        })
        .collect(),
    ]
    .concat()
}

pub async fn get_widget_event_summary_rows(
    db_pool: &PgPool,
    start_date: OffsetDateTime,
    end_date: OffsetDateTime,
) -> Rows {
    vec![
        vec![vec![format!(
            "⎯⎯⎯⎯⎯ {} to {} ⎯⎯⎯⎯⎯",
            format_date(start_date),
            format_date(end_date)
        )]],
        sqlx::query!(
            r#"
            SELECT event_type, count(*)
            FROM widget_events
            WHERE created_at >= $1 and created_at <= $2
            GROUP BY event_type
            ORDER BY event_type
            "#,
            start_date,
            end_date
        )
        .fetch_all(db_pool)
        .await
        .unwrap()
        .iter()
        .map(|widget_event| {
            vec![
                format!(
                    "{} {} to {}",
                    widget_event.event_type,
                    format_date(start_date),
                    format_date(end_date)
                ),
                widget_event.event_type.to_owned(),
                widget_event.count.unwrap().to_string(),
            ]
        })
        .collect(),
    ]
    .concat()
}
