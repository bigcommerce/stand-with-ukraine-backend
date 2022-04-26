use sqlx::{types::time::OffsetDateTime, PgPool};
use uuid::Uuid;

use crate::bigcommerce::BCStore;

#[tracing::instrument(name = "Save store in database", skip(store, pool))]
pub async fn save_store_credentials(store: &BCStore, pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO stores (id, store_hash, access_token, installed_at) 
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (store_hash) DO UPDATE set access_token = $2, installed_at = $4;
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

#[tracing::instrument(name = "Get store credentials from database", skip(store_hash, pool))]
pub async fn get_store_credentials(
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

#[tracing::instrument(name = "Mark store as uninstalled", skip(store_hash, pool))]
pub async fn set_store_as_uninstalled(store_hash: &str, pool: &PgPool) -> Result<(), sqlx::Error> {
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

#[tracing::instrument(name = "Save widget configuration", skip(store_hash, pool))]
pub async fn save_widget_configuration(
    store_hash: &str,
    widget_configuration: &str,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE stores
        SET published = true, widget_configuration = $1
        WHERE store_hash = $2;
        "#,
        serde_json::Value::from(widget_configuration),
        store_hash,
    )
    .execute(pool)
    .await?;

    Ok(())
}
