use anyhow::Result;
use lettre::{
    message::{header::ContentType, Mailbox},
    transport::smtp::{authentication::Credentials, PoolConfig},
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

use crate::config::Settings;

/// Email service for sending notifications
pub struct EmailService {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from_email: Mailbox,
}

impl EmailService {
    pub fn new(_settings: &Settings) -> Result<Self> {
        let smtp_host =
            std::env::var("SMTP_HOST").map_err(|_| anyhow::anyhow!("SMTP_HOST not set"))?;

        let smtp_port = std::env::var("SMTP_PORT")
            .unwrap_or_else(|_| "587".to_string())
            .parse::<u16>()?;

        let smtp_username =
            std::env::var("SMTP_USERNAME").map_err(|_| anyhow::anyhow!("SMTP_USERNAME not set"))?;

        let smtp_password =
            std::env::var("SMTP_PASSWORD").map_err(|_| anyhow::anyhow!("SMTP_PASSWORD not set"))?;

        let from_email_str =
            std::env::var("FROM_EMAIL").map_err(|_| anyhow::anyhow!("FROM_EMAIL not set"))?;

        let credentials = Credentials::new(smtp_username, smtp_password);

        let transport = AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp_host)?
            .port(smtp_port)
            .credentials(credentials)
            .pool_config(PoolConfig::new().max_size(10))
            .build();

        let from_email: Mailbox = from_email_str.parse()?;

        Ok(Self {
            transport,
            from_email,
        })
    }

    pub async fn send_email(
        &self,
        to: &str,
        subject: &str,
        html_body: &str,
        text_body: Option<&str>,
    ) -> Result<()> {
        let to_mailbox: Mailbox = to.parse()?;

        let message = if let Some(text) = text_body {
            Message::builder()
                .from(self.from_email.clone())
                .to(to_mailbox)
                .subject(subject)
                .multipart(
                    lettre::message::MultiPart::alternative()
                        .singlepart(
                            lettre::message::SinglePart::builder()
                                .header(ContentType::TEXT_PLAIN)
                                .body(text.to_string()),
                        )
                        .singlepart(
                            lettre::message::SinglePart::builder()
                                .header(ContentType::TEXT_HTML)
                                .body(html_body.to_string()),
                        ),
                )?
        } else {
            Message::builder()
                .from(self.from_email.clone())
                .to(to_mailbox)
                .subject(subject)
                .singlepart(
                    lettre::message::SinglePart::builder()
                        .header(ContentType::TEXT_HTML)
                        .body(html_body.to_string()),
                )?
        };
        self.transport.send(message).await?;

        Ok(())
    }

    pub async fn send_welcome_email(&self, to: &str, name: &str) -> Result<()> {
        let subject = "Welcome to our platform!";
        let html_body = format!(
            r#"
            <html>
                <body>
                    <h1>Welcome, {}!</h1>
                    <p>Thank you for joining our platform. We're excited to have you on board!</p>
                    <p>Best regards,<br>The Team</p>
                </body>
            </html>
            "#,
            name
        );

        self.send_email(to, subject, &html_body, None).await
    }

    pub async fn send_password_reset_email(&self, to: &str, reset_token: &str) -> Result<()> {
        let subject = "Password Reset Request";
        let reset_url = format!("https://yourapp.com/reset-password?token={}", reset_token);

        let html_body = format!(
            r#"
            <html>
                <body>
                    <h1>Password Reset</h1>
                    <p>You requested a password reset. Click the link below to reset your password:</p>
                    <p><a href="{}">Reset Password</a></p>
                    <p>This link will expire in 1 hour.</p>
                    <p>If you did not request this reset, please ignore this email.</p>
                </body>
            </html>
            "#,
            reset_url
        );

        self.send_email(to, subject, &html_body, None).await
    }

    pub async fn send_email_verification(&self, to: &str, verification_token: &str) -> Result<()> {
        let subject = "Verify your email address";
        let verification_url = format!(
            "https://yourapp.com/verify-email?token={}",
            verification_token
        );

        let html_body = format!(
            r#"
            <html>
                <body>
                    <h1>Email Verification</h1>
                    <p>Please verify your email address by clicking the link below:</p>
                    <p><a href="{}">Verify Email</a></p>
                    <p>This link will expire in 24 hours.</p>
                </body>
            </html>
            "#,
            verification_url
        );

        self.send_email(to, subject, &html_body, None).await
    }
}
