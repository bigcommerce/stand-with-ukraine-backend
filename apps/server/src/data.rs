#![allow(clippy::use_self)] // necessary for enum that uses derive

use anyhow::Context;
use email_address::EmailAddress;
use secrecy::Secret;
use serde::{Deserialize, Serialize};
use sqlx::{types::time::OffsetDateTime, PgPool};
use uuid::Uuid;

use crate::bigcommerce::{script::Script, store::APIToken};

#[tracing::instrument(name = "write store credentials to database", skip(store, pool))]
pub async fn write_store_credentials(store: &APIToken, pool: &PgPool) -> Result<(), sqlx::Error> {
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

#[tracing::instrument(name = "read store credentials from database", skip(store_hash, pool))]
pub async fn read_store_credentials(
    store_hash: &str,
    pool: &PgPool,
) -> Result<APIToken, anyhow::Error> {
    let row = sqlx::query!(
        r#"
        SELECT access_token, store_hash FROM stores WHERE store_hash = $1
        "#,
        store_hash,
    )
    .fetch_one(pool)
    .await?;

    Ok(APIToken::new(
        row.store_hash,
        Secret::from(row.access_token),
    ))
}

#[tracing::instrument(
    name = "write store is uninstalled in database",
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
    name = "write store published status in database",
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

#[tracing::instrument(name = "write unpublish feedback to database", skip(store_hash, pool))]
pub async fn write_unpublish_feedback(
    store_hash: &str,
    reason: &str,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    // get at most the first 1000 characters
    let reason = &reason[0..reason.len().min(1000)];

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

#[derive(Deserialize, Serialize)]
pub struct StoreStatus {
    pub published: bool,
}

#[tracing::instrument(
    name = "read store published status from database",
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

#[derive(Serialize, Deserialize, Debug)]
pub struct WidgetConfiguration {
    pub style: String,
    pub placement: String,
    pub charity_selections: Vec<String>,
    pub modal_title: String,
    pub modal_body: String,
}

impl WidgetConfiguration {
    /// # Errors
    ///
    /// Will return `serde_json::Error` if `&self` cannot be serialized into a string of json.
    pub fn generate_script(
        &self,
        store_hash: &str,
        base_url: &str,
    ) -> Result<Script, serde_json::Error> {
        Ok(Script::new(
         "Stand With Ukraine".to_owned(),
         "This script displays the stand with ukraine widget on your storefront. Configure it from the Stand With Ukraine app installed on your store.".to_owned(),
         format!(
            r#"<script>window.SWU_CONFIG={};window.SWU_CONFIG.store_hash="{}";</script><script src="{}/widget/index.js"></script>"#,
            serde_json::to_string(self)?,
            store_hash,
            base_url
        )))
    }
}

#[tracing::instrument(
    name = "write widget configuration to database",
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
    name = "read widget configuration from database",
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

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum Charity {
    Unicef,
    NewUkraine,
    Razom,
    MiraAction,
}

impl Charity {
    fn to_value_string(&self) -> String {
        serde_json::to_value(self)
            .unwrap()
            .as_str()
            .unwrap()
            .to_owned()
    }
}

#[derive(Deserialize, Debug)]
pub struct CharityEvent {
    store_hash: String,
    charity: Charity,
    event: CharityEventType,
}

#[tracing::instrument(name = "write charity visit event to database", skip(db_pool))]
pub async fn write_charity_visited_event(
    event: &CharityEvent,
    db_pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO charity_events (store_hash, charity, event_type, created_at)
        VALUES ($1, $2, $3, $4);
        "#,
        store_hash_field_from_str(event.store_hash.as_str()),
        event.charity.to_value_string(),
        event.event.to_value_string(),
        OffsetDateTime::now_utc(),
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

#[allow(clippy::use_self)]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum WidgetEventType {
    WidgetOpened,
    WidgetCollapsed,
    WidgetClosed,
    ModalOpened,
    ModalClosed,
}

impl WidgetEventType {
    fn to_value_string(&self) -> String {
        serde_json::to_value(self)
            .unwrap()
            .as_str()
            .unwrap()
            .to_owned()
    }
}

#[allow(clippy::use_self)]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum CharityEventType {
    SupportClicked,
    SeeMoreClicked,
}

impl CharityEventType {
    fn to_value_string(&self) -> String {
        serde_json::to_value(self)
            .unwrap()
            .as_str()
            .unwrap()
            .to_owned()
    }
}

#[derive(Deserialize, Debug)]
pub struct WidgetEvent {
    store_hash: String,
    event: WidgetEventType,
}

pub fn store_hash_field_from_str(store_hash: &str) -> Option<&str> {
    match store_hash {
        "universal" => None,
        store_hash => Some(store_hash),
    }
}

#[tracing::instrument(name = "write widget event to database", skip(db_pool))]
pub async fn write_widget_event(event: &WidgetEvent, db_pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO widget_events (store_hash, event_type, created_at)
        VALUES ($1, $2, $3);
        "#,
        store_hash_field_from_str(event.store_hash.as_str()),
        event.event.to_value_string(),
        OffsetDateTime::now_utc(),
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

#[derive(Deserialize, Debug)]
pub struct FeedbackForm {
    name: String,
    email: EmailAddress,
    message: String,
}

#[tracing::instrument(name = "write feedback to database", skip(db_pool))]
pub async fn write_general_feedback(
    data: &FeedbackForm,
    db_pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO general_feedback (name, email, message, submitted_at)
        VALUES ($1, $2, $3, $4);
        "#,
        data.name.as_str(),
        data.email.as_str(),
        data.message.as_str(),
        OffsetDateTime::now_utc(),
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum UniversalConfiguratorEventType {
    GenerateCode,
    CopyCode,
}

impl UniversalConfiguratorEventType {
    fn to_value_string(&self) -> String {
        serde_json::to_value(self)
            .unwrap()
            .as_str()
            .unwrap()
            .to_owned()
    }
}

#[derive(Deserialize, Debug)]
pub struct UniversalConfiguratorEvent {
    metadata: Option<String>,
    event_type: UniversalConfiguratorEventType,
}

#[tracing::instrument(name = "write universal widget event to database", skip(db_pool))]
pub async fn write_universal_widget_event(
    data: &UniversalConfiguratorEvent,
    db_pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO universal_installer_events (submitted_at, event_type, metadata)
        VALUES ($1, $2, $3);
        "#,
        OffsetDateTime::now_utc(),
        data.event_type.to_value_string(),
        data.metadata
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
    #[case(&WidgetEventType::WidgetOpened, "widget-opened")]
    #[case(&WidgetEventType::WidgetCollapsed, "widget-collapsed")]
    #[case(&WidgetEventType::WidgetClosed, "widget-closed")]
    #[case(&WidgetEventType::ModalOpened, "modal-opened")]
    #[case(&WidgetEventType::ModalClosed, "modal-closed")]
    fn widget_event_type_to_string_works(#[case] event: &WidgetEventType, #[case] value: &str) {
        assert_eq!(event.to_value_string(), value)
    }

    #[rstest]
    #[case(&CharityEventType::SupportClicked, "support-clicked")]
    #[case(&CharityEventType::SeeMoreClicked, "see-more-clicked")]
    fn charity_event_type_to_string_works(#[case] event: &CharityEventType, #[case] value: &str) {
        assert_eq!(event.to_value_string(), value)
    }

    #[rstest]
    #[case(&Charity::NewUkraine, "new-ukraine")]
    #[case(&Charity::Razom, "razom")]
    #[case(&Charity::Unicef, "unicef")]
    #[case(&Charity::MiraAction, "mira-action")]
    fn charity_to_string_works(#[case] charity: &Charity, #[case] value: &str) {
        assert_eq!(charity.to_value_string(), value)
    }

    #[rstest]
    #[case(&UniversalConfiguratorEventType::GenerateCode, "generate-code")]
    #[case(&UniversalConfiguratorEventType::CopyCode, "copy-code")]
    fn universal_configurator_event_to_string_works(
        #[case] event: &UniversalConfiguratorEventType,
        #[case] value: &str,
    ) {
        assert_eq!(event.to_value_string(), value)
    }
}
