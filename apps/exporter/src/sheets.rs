extern crate google_sheets4 as sheets4;
use sheets4::{
    api::ValueRange,
    hyper::{self, client::HttpConnector},
    hyper_rustls::{self, HttpsConnector},
    oauth2, Sheets,
};

pub type Rows = Vec<Vec<String>>;
pub type SheetsClient = Sheets<HttpsConnector<HttpConnector>>;

pub async fn get_sheets_client(credential_path: &str, token_cache_path: &str) -> SheetsClient {
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
                .enable_http2()
                .build(),
        ),
        authenticator,
    )
}

pub async fn create_bulk_updates_for_sheet(
    sheets_client: &SheetsClient,
    spreadsheet_id: &str,
    sheet_name: &str,
    rows: Rows,
) -> Vec<ValueRange> {
    let existing_rows =
        get_existing_rows_from_sheet(sheets_client, spreadsheet_id, sheet_name).await;
    let mut last_row = match &existing_rows {
        Some(rows) => rows.len(),
        None => 0,
    };

    rows.into_iter()
        .map(|new_row| {
            create_update_range_from_row(sheet_name, new_row, &existing_rows, &mut last_row)
        })
        .collect()
}

pub async fn get_existing_rows_from_sheet(
    sheets_client: &SheetsClient,
    spreadsheet_id: &str,
    sheet_name: &str,
) -> Option<Rows> {
    let (_, existing_store_ids) = sheets_client
        .spreadsheets()
        .values_get(spreadsheet_id, format!("{}!A1:A", sheet_name).as_str())
        .doit()
        .await
        .unwrap();

    existing_store_ids.values
}

fn create_update_range_from_row(
    sheet_name: &str,
    new_row: Vec<String>,
    existing_values: &Option<Rows>,
    last_row: &mut usize,
) -> ValueRange {
    let found_row_index = match existing_values {
        Some(rows) => rows.iter().position(|row| {
            if !row.is_empty() {
                row[0] == new_row[0]
            } else {
                false
            }
        }),
        None => None,
    };

    let update_row_index = match found_row_index {
        // 1 based index
        Some(found_row_index) => found_row_index + 1,
        None => {
            *last_row += 1;
            *last_row
        }
    };

    let end_column: char = (b'A' + (new_row.len().max(1) - 1) as u8) as char;

    ValueRange {
        major_dimension: Some("ROWS".to_owned()),
        values: Some(vec![new_row]),
        range: Some(format!(
            "{0}!A{1}:{2}{1}",
            sheet_name, update_row_index, end_column
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_order_update_correctly_from_existing_rows() {
        let sheet_name = "test-sheet";
        let new_row = vec!["1".to_owned(), "test-store".to_owned()];
        let existing_rows = vec![
            vec!["id".to_owned(), "store_hash".to_owned()],
            vec!["1".to_owned(), "old-name".to_owned()],
        ];
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
        let new_row = vec!["1".to_owned(), "test-store-1".to_owned()];
        let existing_rows = vec![
            vec!["id".to_owned(), "store_hash".to_owned()],
            vec!["2".to_owned(), "store-abc".to_owned()],
        ];
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

        let new_row = vec!["3".to_owned(), "test-store-3".to_owned()];

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
