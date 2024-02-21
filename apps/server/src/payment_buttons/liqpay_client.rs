use base64;
use base64::engine::general_purpose::STANDARD as encoder;
use base64::Engine;
use chrono::Local;
use secrecy::{ExposeSecret, Secret};
use sha1::{Digest, Sha1};

use crate::payment_buttons::action::Action;
use crate::payment_buttons::currency::Currency;
use crate::payment_buttons::language::Language;

pub struct LiqPayClient {
    public_key: Secret<String>,
    private_key: Secret<String>,
}

impl LiqPayClient {
    pub fn new(public_key: Secret<String>, private_key: Secret<String>) -> LiqPayClient {
        LiqPayClient {
            public_key,
            private_key,
        }
    }

    pub fn link(
        &self,
        amount: &f64,
        language: &Language,
        currency: &Currency,
        action: &Action,
        description: &str,
    ) -> String {
        let version = "3".to_string();
        let mut params = std::collections::HashMap::new();
        params.insert(
            "public_key".to_string(),
            self.public_key.expose_secret().clone(),
        );
        params.insert("language".to_string(), language.as_string());
        params.insert("version".to_string(), version.clone());
        params.insert("amount".to_string(), amount.to_string());
        params.insert("currency".to_string(), currency.as_string());
        params.insert("description".to_string(), description.to_string());
        params.insert("action".to_string(), action.value());
        params.insert("order_id".to_string(), "".to_string());

        if action.value() == Action::Subscribe.value() {
            params.insert("subscribe".to_string(), "1".to_string());
            params.insert("subscribe_periodicity".to_string(), "month".to_string());
            let current_date_time = Local::now() - chrono::Duration::hours(2);
            let formatted_date_time = current_date_time.format("%Y-%m-%d %H:%M:%S").to_string();
            params.insert("subscribe_date_start".to_string(), formatted_date_time);
        }

        match serde_json::to_string(&params) {
            Err(_) => "#error".to_string(),
            Ok(value) => {
                let data = encoder.encode(value);
                format!(
                    "https://www.liqpay.ua/api/{}/checkout?data={}&signature={}",
                    version,
                    data,
                    self.signature(&data)
                )
            }
        }
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
        let client = LiqPayClient::new(
            Secret::new("public_key".to_string()),
            Secret::new("private_key".to_string()),
        );
        let link = client.link(
            &100.0,
            &Language::UA,
            &Currency::UAH,
            &Action::Subscribe,
            "Stand with Ukraine",
        );
        assert_eq!(
            link.contains("https://www.liqpay.ua/api/3/checkout?data="),
            true
        );
        assert_eq!(link.contains("&signature="), true);
    }
}
