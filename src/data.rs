use anyhow::Context;
use sqlx::{types::time::OffsetDateTime, PgPool};
use uuid::Uuid;

use crate::bigcommerce::BCStore;

#[tracing::instrument(name = "Write store credentials to database", skip(store, pool))]
pub async fn write_store_credentials(store: &BCStore, pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO stores (id, store_hash, access_token, installed_at, uninstalled) 
        VALUES ($1, $2, $3, $4, false)
        ON CONFLICT (store_hash) DO UPDATE set access_token = $3, installed_at = $4, uninstalled = false;
        "#,
        Uuid::new_v4(),
        store.store_hash,
        store.access_token,
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
    let store = sqlx::query_as!(
        BCStore,
        r#"
        SELECT access_token, store_hash FROM stores WHERE store_hash = $1
        "#,
        store_hash,
    )
    .fetch_one(pool)
    .await?;

    Ok(store)
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
        WHERE store_hash = $2;
        "#,
        widget_configuration,
        store_hash,
    )
    .execute(db_pool)
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
