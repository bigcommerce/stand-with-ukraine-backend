use ::time::OffsetDateTime;
use base64::engine::general_purpose::STANDARD as encoder;
use base64::Engine;
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use time::{format_description::FormatItem, macros::format_description};

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
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
pub struct BaseFields {
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
#[serde(untagged)]
pub enum CheckoutRequest {
    Subscription {
        #[serde(flatten)]
        shared: BaseFields,

        subscribe: usize,
        subscribe_periodicity: SubscribePeriod,
        subscribe_date_start: String,
    },

    Pay {
        #[serde(flatten)]
        shared: BaseFields,
    },
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

    #[tracing::instrument(name = "Generate payload", skip(self))]
    pub fn generate_request_payload(
        &self,
        query: InputQuery,
        description: &str,
    ) -> anyhow::Result<CheckoutRequest> {
        let shared = BaseFields {
            public_key: self.public_key.expose_secret().clone(),
            language: query.language,
            action: query.action.clone(),
            version: API_VERSION,
            amount: query.amount,
            currency: query.currency,
            description: description.to_owned(),
            order_id: uuid::Uuid::new_v4().into(),
        };

        Ok(match query.action {
            Action::Subscribe => CheckoutRequest::Subscription {
                shared,
                subscribe: 1,
                subscribe_periodicity: SubscribePeriod::Month,
                subscribe_date_start: OffsetDateTime::now_utc().format(DATE_TIME_FORMAT)?,
            },
            Action::Pay | Action::PayDonate => CheckoutRequest::Pay { shared },
        })
    }

    #[tracing::instrument(name = "Generate LiqPay link", skip(self))]
    pub fn link(&self, request: CheckoutRequest) -> anyhow::Result<String> {
        let data = serde_json::to_string(&request)?;
        let data = encoder.encode(data);

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
    use assert_json_diff::assert_json_include;
    use rstest::rstest;
    use serde_json::json;

    use super::*;

    #[test]
    fn test_create_link_subscribe() {
        let client = HttpAPI::new(
            Secret::new("public_key".to_string()),
            Secret::new("private_key".to_string()),
        );

        let checkout_request = client
            .generate_request_payload(
                InputQuery {
                    amount: 100.0,
                    language: Language::UA,
                    currency: Currency::UAH,
                    action: Action::Subscribe,
                },
                "Stand with Ukraine",
            )
            .unwrap();

        let link = client.link(checkout_request).unwrap();

        assert_eq!(
            link.starts_with("https://www.liqpay.ua/api/3/checkout?data="),
            true
        );
        assert_eq!(link.contains("&signature="), true);
    }

    #[test]
    fn test_subscribe_request_serialization() {
        let request = CheckoutRequest::Subscription {
            shared: BaseFields {
                public_key: "public_key".to_owned(),
                language: Language::UA,
                action: Action::Subscribe,
                version: 3,
                amount: 100.00,
                currency: Currency::USD,
                description: "stand with ukraine".to_owned(),
                order_id: "1234".to_owned(),
            },
            subscribe: 1,
            subscribe_periodicity: SubscribePeriod::Month,
            subscribe_date_start: "2024-02-23 15:26:00".to_owned(),
        };

        assert_json_include!(
            actual: serde_json::to_value(&request).unwrap(),
            expected: json!({
                "public_key":"public_key",
                "language":"ua",
                "action":"subscribe",
                "version":3,
                "amount":100.0,
                "currency":"USD",
                "description":"stand with ukraine",
                "order_id":"1234",
                "subscribe":1,
                "subscribe_periodicity":"month",
                "subscribe_date_start":"2024-02-23 15:26:00"
            })
        );
    }
    #[test]
    fn test_donate_request_serialization() {
        let request = CheckoutRequest::Pay {
            shared: BaseFields {
                public_key: "public_key".to_owned(),
                language: Language::UA,
                action: Action::PayDonate,
                version: 3,
                amount: 100.00,
                currency: Currency::USD,
                description: "stand with ukraine".to_owned(),
                order_id: "1234".to_owned(),
            },
        };

        assert_json_include!(
            actual: serde_json::to_value(&request).unwrap(),
            expected: json!({
                "public_key":"public_key",
                "language":"ua",
                "action":"paydonate",
                "version":3,
                "amount":100.0,
                "currency":"USD",
                "description":"stand with ukraine",
                "order_id":"1234"
            })
        );
    }

    #[test]
    fn test_create_link_pay() {
        let client = HttpAPI::new(
            Secret::new("public_key".to_string()),
            Secret::new("private_key".to_string()),
        );

        let checkout_request = client
            .generate_request_payload(
                InputQuery {
                    amount: 100.0,
                    language: Language::EN,
                    currency: Currency::USD,
                    action: Action::Pay,
                },
                "Stand with Ukraine",
            )
            .unwrap();

        let link = client.link(checkout_request).unwrap();

        assert_eq!(
            link.starts_with("https://www.liqpay.ua/api/3/checkout?data="),
            true
        );
        assert_eq!(link.contains("&signature="), true);
    }

    #[rstest]
    #[case(Action::Pay, "pay")]
    #[case(Action::Subscribe, "subscribe")]
    #[case(Action::PayDonate, "paydonate")]
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
