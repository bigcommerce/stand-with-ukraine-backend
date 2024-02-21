#[derive(Debug, Eq, PartialEq)]
pub enum Currency {
    USD,
    EUR,
    UAH,
}
impl Currency {
    pub fn as_string(&self) -> String {
        match self {
            Currency::USD => "USD".to_string(),
            Currency::EUR => "EUR".to_string(),
            Currency::UAH => "UAH".to_string(),
        }
    }
}

impl Currency {
    pub fn new(currency: &str) -> Currency {
        match currency.to_lowercase().as_str() {
            "eur" => Currency::EUR,
            "uah" => Currency::UAH,
            _ => Currency::USD,
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(Currency::USD, "USD")]
    #[case(Currency::EUR, "EUR")]
    #[case(Currency::UAH, "UAH")]
    fn test_currency_to_string(#[case] currency: Currency, #[case] currency_string: &str) {
        assert_eq!(currency_string, currency.as_string());
    }

    #[rstest]
    #[case(Currency::USD, "usd")]
    #[case(Currency::EUR, "EUR")]
    #[case(Currency::UAH, "uah")]
    #[case(Currency::USD, "tt")]
    fn test_currency_new(#[case] currency: Currency, #[case] currency_string: &str) {
        assert_eq!(Currency::new(currency_string), currency);
    }
}
