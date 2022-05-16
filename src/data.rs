use anyhow::Context;
use secrecy::Secret;
use sqlx::{types::time::OffsetDateTime, PgPool};
use uuid::Uuid;

use crate::{
    bigcommerce::{script::Script, store::BCStore},
    configuration::ApplicationBaseUrl,
};

#[tracing::instrument(name = "Write store credentials to database", skip(store, pool))]
pub async fn write_store_credentials(store: &BCStore, pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO stores (id, store_hash, access_token, installed_at, uninstalled)
        VALUES ($1, $2, $3, $4, false)
        ON CONFLICT (store_hash) DO UPDATE set access_token = $3, installed_at = $4, uninstalled = false;
        "#,
        Uuid::new_v4(),
        store.get_store_hash(),
        store.get_access_token(),
        OffsetDateTime::now_utc()
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[tracing::instrument(name = "Read store credentials from database", skip(store_hash, pool))]
pub async fn read_store_credentials(
    store_hash: &str,
    pool: &PgPool,
) -> Result<BCStore, anyhow::Error> {
    let row = sqlx::query!(
        r#"
        SELECT access_token, store_hash FROM stores WHERE store_hash = $1
        "#,
        store_hash,
    )
    .fetch_one(pool)
    .await?;

    Ok(BCStore::new(row.store_hash, Secret::from(row.access_token)))
}

#[tracing::instrument(
    name = "Write store is uninstalled in database",
    skip(store_hash, pool)
)]
pub async fn write_store_as_uninstalled(
    store_hash: &str,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE stores
        SET uninstalled = true, published = false
        WHERE store_hash = $1;
        "#,
        store_hash,
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[tracing::instrument(
    name = "Write store published status in database",
    skip(store_hash, pool)
)]
pub async fn write_store_published(
    store_hash: &str,
    status: bool,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE stores
        SET published = $1
        WHERE store_hash = $2;
        "#,
        status,
        store_hash,
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[tracing::instrument(name = "Write unpublish feedback to database", skip(store_hash, pool))]
pub async fn write_unpublish_feedback(
    store_hash: &str,
    reason: &str,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO unpublish_events (store_hash, unpublished_at, reason)
        VALUES ($1, $2, $3);
        "#,
        store_hash,
        OffsetDateTime::now_utc(),
        reason
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct StoreStatus {
    pub published: bool,
}

#[tracing::instrument(
    name = "Read store published status from database",
    skip(store_hash, pool)
)]
pub async fn read_store_published(
    store_hash: &str,
    pool: &PgPool,
) -> Result<StoreStatus, sqlx::Error> {
    let store_status = sqlx::query_as!(
        StoreStatus,
        r#"
        SELECT published FROM stores
        WHERE store_hash = $1;
        "#,
        store_hash,
    )
    .fetch_one(pool)
    .await?;

    Ok(store_status)
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct WidgetConfiguration {
    pub style: String,
    pub placement: String,
    pub charity_selections: Vec<String>,
    pub modal_title: String,
    pub modal_body: String,
}

impl WidgetConfiguration {
    pub fn generate_script(
        &self,
        base_url: &ApplicationBaseUrl,
    ) -> Result<Script, serde_json::Error> {
        Ok(Script::new(
         "Stand With Ukraine".to_owned(),
         "This script displays the stand with ukraine widget on your storefront. Configure it from the Stand With Ukraine app installed on your store.".to_owned(),
         format!(
            r#"<script>window.SWU_CONFIG={};</script><script src="{}/widget/index.js"></script>"#,
            serde_json::to_string(self)?,
            base_url
        )))
    }
}

#[tracing::instrument(
    name = "Write widget configuration to database",
    skip(store_hash, db_pool)
)]
pub async fn write_widget_configuration(
    store_hash: &str,
    widget_configuration: &WidgetConfiguration,
    db_pool: &PgPool,
) -> Result<(), anyhow::Error> {
    let widget_configuration =
        serde_json::value::to_value(widget_configuration).context("Convert to json value")?;

    sqlx::query!(
        r#"
        UPDATE stores
        SET widget_configuration = $1
        WHERE store_hash = $2
        RETURNING id
        "#,
        widget_configuration,
        store_hash,
    )
    .fetch_one(db_pool)
    .await
    .context("Save configuration to database")?;

    Ok(())
}

#[tracing::instrument(
    name = "Read widget configuration from database",
    skip(store_hash, db_pool)
)]
pub async fn read_widget_configuration(
    store_hash: &str,
    db_pool: &PgPool,
) -> Result<WidgetConfiguration, anyhow::Error> {
    let row = sqlx::query!(
        r#"
        SELECT widget_configuration FROM stores
        WHERE store_hash = $1;
        "#,
        store_hash,
    )
    .fetch_one(db_pool)
    .await
    .context("Save configuration to database")?;

    let widget_configuration: WidgetConfiguration =
        serde_json::value::from_value(row.widget_configuration)
            .context("Parse database json to application format")?;

    Ok(widget_configuration)
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Charity {
    Unicef,
    NewUkraine,
    Razom,
}

#[tracing::instrument(
    name = "Write charity visit event to database",
    skip(store_hash, charity, db_pool)
)]
pub async fn write_charity_visited_event(
    store_hash: &str,
    charity: &Charity,
    db_pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO charity_visited_events (store_hash, charity, created_at)
        VALUES ($1, $2, $3);
        "#,
        store_hash,
        serde_json::to_string(charity).unwrap(),
        OffsetDateTime::now_utc(),
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WidgetEventType {
    Opened,
    Collapsed,
    Closed,
}

#[tracing::instrument(
    name = "Write widget event to database",
    skip(store_hash, db_pool, event_type)
)]
pub async fn write_widget_event(
    store_hash: &str,
    event_type: &WidgetEventType,
    db_pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO widget_events (store_hash, event_type, created_at)
        VALUES ($1, $2, $3);
        "#,
        store_hash,
        serde_json::to_string(event_type).unwrap(),
        OffsetDateTime::now_utc(),
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case(&WidgetEventType::Opened, "opened")]
    #[case(&WidgetEventType::Collapsed, "collapsed")]
    #[case(&WidgetEventType::Closed, "closed")]
    fn widget_event_type_to_string_works(#[case] event: &WidgetEventType, #[case] value: &str) {
        assert_eq!(serde_variant::to_variant_name(event).unwrap(), value)
    }

    #[rstest]
    #[case(&Charity::NewUkraine, "new-ukraine")]
    #[case(&Charity::Razom, "razom")]
    #[case(&Charity::Unicef, "unicef")]
    fn charity_to_string_works(#[case] charity: &Charity, #[case] value: &str) {
        assert_eq!(serde_variant::to_variant_name(charity).unwrap(), value)
    }
}
