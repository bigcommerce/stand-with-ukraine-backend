use std::sync::Arc;

use axum::extract::FromRef;
use secrecy::Secret;
use sqlx::PgPool;

use crate::{
    bigcommerce::client::HttpAPI as BigCommerceHttpAPI, liq_pay::HttpAPI as LiqPayHttpAPI,
};

#[allow(clippy::module_name_repetitions)]
// reason="`AppState` is clearer than just `App` and it is widespread across the app"
#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub base_url: String,
    pub jwt_secret: Secret<String>,
    pub bigcommerce_client: BigCommerceHttpAPI,
    pub liq_pay_client: LiqPayHttpAPI,
}

#[allow(clippy::module_name_repetitions)]
// reason="`SharedState` is clearer than just `Shared` and it is widespread across the app"
pub type SharedState = Arc<AppState>;

impl FromRef<SharedState> for AppState {
    fn from_ref(shared_state: &SharedState) -> Self {
        shared_state.as_ref().clone()
    }
}
