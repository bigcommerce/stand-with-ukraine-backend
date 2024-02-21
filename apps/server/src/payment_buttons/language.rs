#[derive(Debug, PartialEq, Eq)]
pub enum Language {
    UA,
    EN,
}

impl Language {
    pub fn as_string(&self) -> String {
        match self {
            Language::UA => "ua".to_string(),
            Language::EN => "en".to_string(),
        }
    }
    pub fn new(language: &str) -> Language {
        match language {
            "ua" => Language::UA,
            _ => Language::EN,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(Language::UA, "ua")]
    #[case(Language::EN, "en")]
    fn test_language_to_string(#[case] language: Language, #[case] language_string: &str) {
        assert_eq!(language.as_string(), language_string);
    }

    #[rstest]
    #[case(Language::UA, "ua")]
    #[case(Language::EN, "en")]
    #[case(Language::EN, "tt")]
    fn test_language_new(#[case] language: Language, #[case] language_string: &str) {
        assert_eq!(Language::new(language_string), language);
    }
}
