use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(s: String) -> Result<SubscriberName, String> {
        let is_empty = s.trim().is_empty();
        let is_too_long = s.graphemes(true).count() > 256;
        let forbidden_characters = [ '(', ')', '/', '"', '<', '>', '}', '{', '}', '\\' ];
        let has_forbidden_characters = s.chars().any(|c| forbidden_characters.contains(&c));
        if is_empty || is_too_long || has_forbidden_characters {
            Err("Invalid subscriber name.".into())
        } else {
            Ok(Self(s))
        }
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberName;
    use claims::{assert_err, assert_ok};

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "à".repeat(256);
        assert_ok!(SubscriberName::parse(name));
    }

    #[test]
    fn a_name_longer_than_256_is_rejected() {
        let name = "u".repeat(257);
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn whitespace_only_is_rejected() {
        let name = " ";
        assert_err!(SubscriberName::parse(name.into()));
    }

    #[test]
    fn empty_string_is_rejected() {
        let name = "";
        assert_err!(SubscriberName::parse(name.into()));
    }

    #[test]
    fn name_with_forbidden_characters_is_rejected() {
        let name = "name with (parenthesis)";
        assert_err!(SubscriberName::parse(name.into()));
    }
}