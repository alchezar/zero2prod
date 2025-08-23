use crate::lib::configurations::EmailClientSettings;
use crate::lib::domain::SubscriberEmail;
use actix_web::mime::APPLICATION_JSON;
use reqwest::header::CONTENT_TYPE;
use secrecy::{ExposeSecret, SecretString};

#[derive(serde::Serialize)]
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
            .header(
                "X-Postmark-Server-Token",
                self.authorization_token.expose_secret(),
            )
            .json(&request_body)
            .send()
            .await?;
        Ok(())
    }

    pub fn new(
        base_url: String,
        sender: SubscriberEmail,
        authorization_token: SecretString,
    ) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            base_url,
            sender,
            authorization_token,
        }
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
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::lib::domain::SubscriberEmail;
    use crate::lib::email_client::EmailClient;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::Sentence;
    use fake::{Fake, Faker};
    use secrecy::SecretString;
    use wiremock::matchers::any;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        // Arrange
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::parse(&SafeEmail().fake::<String>()).unwrap();
        let email_client = EmailClient::new(
            mock_server.uri(),
            sender,
            SecretString::new(Faker.fake::<String>().into()),
        );

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(&SafeEmail().fake::<String>()).unwrap();
        let subject = Sentence(1..2).fake::<String>();
        let content = Sentence(1..10).fake::<String>();

        // Act
        let _ = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;
    }
}
