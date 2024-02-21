use serde::Serialize;

use crate::payment_buttons::action::Action;
use crate::payment_buttons::currency::Currency;
use crate::payment_buttons::language::Language;
use crate::payment_buttons::liqpay_client::LiqPayClient;

#[derive(Serialize)]
struct Link {
    name: String,
    url: String,
}

#[derive(Serialize)]
pub(crate) struct Links {
    subscription: Vec<Link>,
    once: Link,
}

impl Links {
    pub fn new(
        language: &Language,
        currency: &Currency,
        sum: &[f64],
        links: &LiqPayClient,
    ) -> Links {
        return Links {
            subscription: sum
                .iter()
                .map(|amount| Link {
                    name: format!("{} {}", amount, currency.as_string().to_uppercase()),
                    url: links.link(
                        amount,
                        language,
                        currency,
                        &Action::Subscribe,
                        "Stand with Ukraine",
                    ),
                })
                .collect::<Vec<Link>>(),
            once: Link {
                name: match language {
                    Language::UA => "Інша сума".to_string(),
                    _ => "Custom".to_string(),
                },
                url: links.link(
                    &100.0,
                    language,
                    currency,
                    &Action::PayDonate,
                    "Support BigCommerce colleagues defending Ukraine",
                ),
            },
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::payment_buttons::liqpay_client::LiqPayClient;
    use rstest::rstest;
    use secrecy::Secret;

    use super::*;

    #[rstest]
    #[case(Language::UA, Currency::UAH, vec![100.0, 200.0], 2)]
    #[case(Language::EN, Currency::USD, vec![100.0, 100.0, 3.0], 3)]
    fn test_create_links(
        #[case] language: Language,
        #[case] currency: Currency,
        #[case] sum: Vec<f64>,
        #[case] num: usize,
    ) {
        let links = LiqPayClient::new(
            Secret::new("public_key".to_string()),
            Secret::new("private_key".to_string()),
        );
        let expect_text = match language {
            Language::UA => "Інша сума".to_string(),
            _ => "Custom".to_string(),
        };
        let result = Links::new(&language, &currency, &sum, &links);
        assert_eq!(result.subscription.len(), num);
        assert_eq!(result.once.name, expect_text);
    }
}
