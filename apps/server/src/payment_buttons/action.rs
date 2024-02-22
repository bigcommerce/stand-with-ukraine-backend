#[derive(Debug, Eq, PartialEq)]
pub enum Action {
    Pay,
    Subscribe,
    PayDonate,
}

impl Action {
    pub fn new(value: &str) -> Action {
        match value {
            "pay" => Action::Pay,
            "subscribe" => Action::Subscribe,
            _ => Action::PayDonate,
        }
    }
    pub fn value(&self) -> String {
        match self {
            Action::Pay => "pay".to_string(),
            Action::Subscribe => "subscribe".to_string(),
            Action::PayDonate => "paydonate".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(Action::Pay, "pay")]
    #[case(Action::Subscribe, "subscribe")]
    #[case(Action::PayDonate, "paydonate")]
    fn test_action_to_string(#[case] action: Action, #[case] action_string: &str) {
        assert_eq!(action.value(), action_string);
    }
    #[rstest]
    #[case(Action::Pay, "pay")]
    #[case(Action::Subscribe, "subscribe")]
    #[case(Action::PayDonate, "paydonate")]
    #[case(Action::PayDonate, "tt")]
    fn test_action_new(#[case] action: Action, #[case] action_string: &str) {
        assert_eq!(Action::new(action_string), action);
    }
}
