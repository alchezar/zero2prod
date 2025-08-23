use validator::ValidateEmail;

#[derive(Debug, Clone)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(email: &str) -> Result<SubscriberEmail, String> {
        if email.validate_email() {
            Ok(Self(email.into()))
        } else {
            Err(format!("{} is not a valid subscriber email", email))
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::assert_err;
    use fake::Fake;
    use fake::faker::internet::en::SafeEmail;

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_owned();
        assert_err!(SubscriberEmail::parse(&email));
    }

    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "ursula.domain.com".to_owned();
        assert_err!(SubscriberEmail::parse(&email));
    }

    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@domain.com".to_owned();
        assert_err!(SubscriberEmail::parse(&email));
    }

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);
    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary(_g: &mut quickcheck::Gen) -> Self {
            Self(SafeEmail().fake_with_rng(&mut rand::rng()))
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_successfully(valid_email: ValidEmailFixture) {
        dbg!(&valid_email);
        let _ = SubscriberEmail::parse(&valid_email.0).is_ok();
    }
}
