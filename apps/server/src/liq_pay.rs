use ::time::{Duration, OffsetDateTime};
use base64::engine::general_purpose::STANDARD as encoder;
use base64::Engine;
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use time::{format_description::FormatItem, macros::format_description};
use tracing::debug;

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Action {
    Pay,
    Subscribe,
    PayDonate,
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum Currency {
    USD,
    EUR,
    UAH,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    UA,
    EN,
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SubscribePeriod {
    Month,
}

#[derive(Debug, Deserialize)]
pub struct InputQuery {
    pub language: Language,
    pub currency: Currency,
    pub amount: f64,
    pub action: Action,
}

pub struct HttpAPI {
    public_key: Secret<String>,
    private_key: Secret<String>,
}

#[derive(Debug, Serialize)]
pub struct Payload {
    public_key: String,
    language: Language,
    action: Action,
    version: usize,
    amount: f64,
    currency: Currency,
    description: String,
    order_id: String,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionPayload {
    #[serde(flatten)]
    payload: Payload,

    subscribe: usize,
    subscribe_periodically: SubscribePeriod,
    subscribe_date_start: String,
}

const API_VERSION: usize = 3;
const DATE_TIME_FORMAT: &[FormatItem<'_>] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

impl HttpAPI {
    pub fn new(public_key: Secret<String>, private_key: Secret<String>) -> Self {
        Self {
            public_key,
            private_key,
        }
    }

    #[tracing::instrument(name = "Generate LiqPay link", skip(self))]
    pub fn link(&self, query: InputQuery, description: &str) -> anyhow::Result<String> {
        let payload = Payload {
            public_key: self.public_key.expose_secret().clone(),
            language: query.language,
            action: query.action.clone(),
            version: API_VERSION,
            amount: query.amount,
            currency: query.currency,
            description: description.to_owned(),
            order_id: uuid::Uuid::new_v4().into(),
        };

        let payload = match query.action {
            Action::Subscribe => serde_json::to_string(&SubscriptionPayload {
                payload,
                subscribe: 1,
                subscribe_periodically: SubscribePeriod::Month,
                subscribe_date_start: (OffsetDateTime::now_utc() - Duration::hours(2))
                    .format(DATE_TIME_FORMAT)?,
            })?,
            _ => serde_json::to_string(&payload)?,
        };

        debug!(payload, "generated payload");

        let data = encoder.encode(payload);
        Ok(format!(
            "https://www.liqpay.ua/api/{}/checkout?data={}&signature={}",
            API_VERSION,
            data,
            self.signature(&data)
        ))
    }

    fn signature(&self, data: &String) -> String {
        let mut hasher = Sha1::new();
        hasher.update(format!(
            "{}{}{}",
            self.private_key.expose_secret(),
            &data,
            self.private_key.expose_secret()
        ));
        encoder.encode(hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    fn test_create_link() {
        let client = HttpAPI::new(
            Secret::new("public_key".to_string()),
            Secret::new("private_key".to_string()),
        );

        let link = client
            .link(
                InputQuery {
                    amount: 100.0,
                    language: Language::UA,
                    currency: Currency::UAH,
                    action: Action::Subscribe,
                },
                "Stand with Ukraine",
            )
            .unwrap();

        assert_eq!(
            link.contains("https://www.liqpay.ua/api/3/checkout?data="),
            true
        );
        assert_eq!(link.contains("&signature="), true);
    }

    #[rstest]
    #[case(Action::Pay, "pay")]
    #[case(Action::Subscribe, "subscribe")]
    #[case(Action::PayDonate, "pay-donate")]
    fn test_action_new(#[case] action: Action, #[case] action_string: &str) {
        assert_eq!(
            serde_json::from_value::<Action>(action_string.into()).unwrap(),
            action
        );
    }

    #[rstest]
    #[case(Language::UA, "ua")]
    #[case(Language::EN, "en")]
    fn test_language_new(#[case] language: Language, #[case] language_string: &str) {
        assert_eq!(
            serde_json::from_value::<Language>(language_string.into()).unwrap(),
            language
        );
    }
}
