use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(s: String) -> Result<SubscriberName, String> {
        let is_empty_or_whitespace = s.trim().is_empty();
        let is_too_long = s.graphemes(true).count() > 256;

        let forbidden_characters = ['/', '(', ')', '"', '\'', '<', '>', '\\', '{', '}', '[', ']'];
        let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            Err(format!("{} is not a valid subscriber name", s))
        } else {
            Ok(Self(s))
        }
    }

    pub fn inner(self) -> String {
        self.0
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsMut<str> for SubscriberName {
    fn as_mut(&mut self) -> &mut str {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberName;

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "a".repeat(256);
        assert!(SubscriberName::parse(name).is_ok());
    }

    #[test]
    fn a_name_longer_than_256_is_rejected() {
        let name = "a".repeat(257);
        assert!(SubscriberName::parse(name).is_err());
    }

    #[test]
    fn whitespace_only_name_is_rejected() {
        let name = "    ".to_string();
        assert!(SubscriberName::parse(name).is_err());
    }

    #[test]
    fn names_containing_an_invalid_character_are_rejected() {
        let invalid_characters = ['/', '(', ')', '"', '\'', '<', '>', '\\', '{', '}', '[', ']'];
        for name in &invalid_characters {
            assert!(SubscriberName::parse(name.to_string()).is_err())
        }
    }

    #[test]
    fn a_valid_name_is_parsed_successfully() {
        let name = "Ursula Le Guin".to_string();
        assert!(SubscriberName::parse(name).is_ok());
    }
}
