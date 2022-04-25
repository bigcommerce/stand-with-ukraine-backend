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
