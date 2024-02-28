use std::sync::Arc;

use axum::extract::FromRef;
use secrecy::Secret;
use sqlx::PgPool;

use crate::{
    bigcommerce::client::HttpAPI as BigCommerceHttpAPI, liq_pay::HttpAPI as LiqPayHttpAPI,
};

#[derive(Clone)]
pub struct App {
    pub db_pool: PgPool,
    pub base_url: String,
    pub jwt_secret: Secret<String>,
    pub bigcommerce_client: BigCommerceHttpAPI,
    pub liq_pay_client: LiqPayHttpAPI,
}

pub type Shared = Arc<App>;

impl FromRef<Shared> for App {
    fn from_ref(shared_state: &Shared) -> Self {
        shared_state.as_ref().clone()
    }
}
