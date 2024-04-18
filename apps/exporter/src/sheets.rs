extern crate google_sheets4 as sheets4;
use serde_json::Value;
use sheets4::{
    api::ValueRange,
    hyper::{self, client::HttpConnector},
    hyper_rustls::{self, HttpsConnector},
    oauth2, Sheets,
};

pub type Rows = Vec<Vec<Value>>;
pub type Client = Sheets<HttpsConnector<HttpConnector>>;

pub async fn get_sheets_client(credential_path: &str, token_cache_path: &str) -> Client {
    let service_account_key = oauth2::read_service_account_key(credential_path)
        .await
        .expect("failed to read service account");

    let authenticator = oauth2::ServiceAccountAuthenticator::builder(service_account_key)
        .persist_tokens_to_disk(token_cache_path)
        .build()
        .await
        .expect("failed to create authenticator");

    Sheets::new(
        hyper::Client::builder().build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .https_or_http()
                .enable_http1()
                .build(),
        ),
        authenticator,
    )
}

pub async fn create_bulk_updates_for_sheet(
    sheets_client: &Client,
    spreadsheet_id: &str,
    sheet_name: &str,
    rows: Rows,
) -> Vec<ValueRange> {
    let existing_rows =
        get_existing_rows_from_sheet(sheets_client, spreadsheet_id, sheet_name).await;
    let mut last_row = existing_rows.as_ref().map_or(0, std::vec::Vec::len);

    rows.into_iter()
        .map(|new_row| {
            create_update_range_from_row(sheet_name, new_row, &existing_rows, &mut last_row)
        })
        .collect()
}

pub async fn get_existing_rows_from_sheet(
    sheets_client: &Client,
    spreadsheet_id: &str,
    sheet_name: &str,
) -> Option<Rows> {
    let (_, existing_store_ids) = sheets_client
        .spreadsheets()
        .values_get(spreadsheet_id, format!("{sheet_name}!A1:A").as_str())
        .doit()
        .await
        .unwrap();

    existing_store_ids.values
}

#[allow(clippy::cast_possible_truncation)]
fn create_update_range_from_row(
    sheet_name: &str,
    new_row: Vec<Value>,
    existing_values: &Option<Rows>,
    last_row: &mut usize,
) -> ValueRange {
    let found_row_index = existing_values.as_ref().and_then(|rows| {
        rows.iter().position(|row| {
            if row.is_empty() {
                false
            } else {
                row[0] == new_row[0]
            }
        })
    });

    let update_row_index = found_row_index.map_or_else(
        || {
            *last_row += 1;
            *last_row
        },
        |found_row_index| found_row_index + 1,
    );

    let end_column: char = (b'A' + (new_row.len().max(1) - 1) as u8) as char;

    ValueRange {
        major_dimension: Some("ROWS".to_owned()),
        values: Some(vec![new_row]),
        range: Some(format!(
            "{sheet_name}!A{update_row_index}:{end_column}{update_row_index}"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_order_update_correctly_from_existing_rows() {
        let sheet_name = "test-sheet";
        let new_row: Vec<Value> = ["1".to_owned(), "test-store".to_owned()]
            .into_iter()
            .map(Into::into)
            .collect();
        let existing_rows: Vec<Vec<Value>> = [
            ["id", "store_hash"].into_iter().map(Into::into).collect(),
            ["1", "old-name"].into_iter().map(Into::into).collect(),
        ]
        .into();
        let mut last_row = existing_rows.len();

        let updates = create_update_range_from_row(
            sheet_name,
            new_row.clone(),
            &Some(existing_rows),
            &mut last_row,
        );

        assert_eq!(updates.major_dimension, Some("ROWS".to_owned()),);
        assert_eq!(updates.values, Some(vec![new_row]),);
        assert_eq!(updates.range, Some("test-sheet!A2:B2".to_owned()));
    }

    #[test]
    fn update_should_add_to_end_if_match_not_found() {
        let sheet_name = "test-sheet";
        let new_row: Vec<Value> = ["1", "test-store-1"].into_iter().map(Into::into).collect();
        let existing_rows: Vec<Vec<Value>> = [
            ["id", "store_hash"].into_iter().map(Into::into).collect(),
            ["2", "store-abc"].into_iter().map(Into::into).collect(),
        ]
        .into();
        let mut last_row = existing_rows.len();

        let updates = create_update_range_from_row(
            sheet_name,
            new_row.clone(),
            &Some(existing_rows.clone()),
            &mut last_row,
        );

        assert_eq!(updates.major_dimension, Some("ROWS".to_owned()),);
        assert_eq!(updates.values, Some(vec![new_row]),);
        assert_eq!(updates.range, Some("test-sheet!A3:B3".to_owned()));
        assert_eq!(last_row, existing_rows.len() + 1);

        let new_row: Vec<Value> = ["3", "test-store-3"].into_iter().map(Into::into).collect();

        let updates = create_update_range_from_row(
            sheet_name,
            new_row.clone(),
            &Some(existing_rows.clone()),
            &mut last_row,
        );

        assert_eq!(updates.major_dimension, Some("ROWS".to_owned()),);
        assert_eq!(updates.values, Some(vec![new_row]),);
        assert_eq!(updates.range, Some("test-sheet!A4:B4".to_owned()));
        assert_eq!(last_row, existing_rows.len() + 2);
    }
}
