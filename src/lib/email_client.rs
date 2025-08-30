use crate::lib::configurations::EmailClientSettings;
use crate::lib::domain::SubscriberEmail;
use actix_web::mime::APPLICATION_JSON;
use reqwest::header::CONTENT_TYPE;
use secrecy::{ExposeSecret, SecretString};

pub const POSTMARK_HEADER: &'static str = "X-Postmark-Server-Token";
#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
}

#[derive(Clone)]
pub struct EmailClient {
    http_client: reqwest::Client,
    base_url: String,
    sender: SubscriberEmail,
    authorization_token: SecretString,
}

impl EmailClient {
    pub fn new(
        base_url: String,
        sender: SubscriberEmail,
        authorization_token: SecretString,
        timeout: std::time::Duration,
    ) -> Self {
        let http_client = reqwest::Client::builder().timeout(timeout).build().unwrap();

        Self {
            http_client,
            base_url,
            sender,
            authorization_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/email", self.base_url);
        let request_body = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject,
            html_body: html_content,
            text_body: text_content,
        };

        self.http_client
            .post(url)
            .header(CONTENT_TYPE, APPLICATION_JSON.to_string())
            .header(POSTMARK_HEADER, self.authorization_token.expose_secret())
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

impl TryFrom<EmailClientSettings> for EmailClient {
    type Error = String;
    fn try_from(value: EmailClientSettings) -> Result<Self, Self::Error> {
        let sender_email = value.sender()?;
        Ok(Self::new(
            value.base_url,
            sender_email,
            value.authorization_token,
            std::time::Duration::from_millis(value.timeout_ms),
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::lib::domain::SubscriberEmail;
    use crate::lib::email_client::{EmailClient, POSTMARK_HEADER};
    use actix_web::mime::APPLICATION_JSON;
    use claims::{assert_err, assert_ok};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::Sentence;
    use fake::{Fake, Faker};
    use reqwest::header::CONTENT_TYPE;
    use secrecy::SecretString;
    use wiremock::matchers::{any, header, header_exists, method, path};
    use wiremock::{Match, Mock, MockServer, Request, ResponseTemplate};

    struct SendEmailBodyMatcher;
    impl Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            // Try to parse the body as s JSON value.
            let result = serde_json::from_slice::<'_, serde_json::Value>(&request.body);
            if let Ok(body) = result {
                dbg!(&body);
                // Check that all the mandatory fields are populated without
                // inspecting the field values.
                body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("Subject").is_some()
                    && body.get("HtmlBody").is_some()
                    && body.get("TextBody").is_some()
            } else {
                // If parsing failed, ndo not match the request.
                false
            }
        }
    }

    fn subject() -> String {
        Sentence(1..2).fake()
    }
    fn content() -> String {
        Sentence(1..10).fake()
    }
    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(&SafeEmail().fake::<String>()).unwrap()
    }
    fn email_client(base_url: &str) -> EmailClient {
        EmailClient::new(
            base_url.into(),
            email(),
            SecretString::new(Faker.fake::<String>().into()),
            std::time::Duration::from_millis(200),
        )
    }

    #[tokio::test]
    async fn send_email_sends_the_expected_request() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(&mock_server.uri());

        Mock::given(header_exists(POSTMARK_HEADER))
            .and(header(CONTENT_TYPE, APPLICATION_JSON.to_string()))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let _ = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;
    }

    #[tokio::test]
    async fn send_email_succeeds_if_the_server_returns_200() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(&mock_server.uri());

        // We don't copy in all the matchers we have in the other test. The
        // purpose of this test is not to assert on the request we are sending
        // out!.
        // We add the bare minimum needed to trigger the path we want to test in
        // `send_email`.
        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        // Assert
        assert_ok!(outcome);
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(&mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        assert_err!(outcome);
    }

    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(&mock_server.uri());

        let response = ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(180));
        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        assert_err!(outcome);
    }
}
