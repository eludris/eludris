use lettre::{message::SinglePart, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use tokio::fs;

use crate::{conf::Email, models::ErrorResponse};

pub struct Emailer(pub Option<AsyncSmtpTransport<Tokio1Executor>>);

#[derive(Debug, Clone, PartialEq)]
pub enum EmailPreset {
    Verify { code: u32 },
}

impl Emailer {
    pub async fn send_email(
        &self,
        to: &str,
        preset: EmailPreset,
        email: &Email,
    ) -> Result<(), ErrorResponse> {
        let (subject, content) = match preset {
            EmailPreset::Verify { code } => {
                let code = code
                    .to_string()
                    .chars()
                    .collect::<Vec<char>>()
                    .chunks(3)
                    .map(|c| c.iter().collect::<String>())
                    .collect::<Vec<String>>()
                    .join(" ");
                let content = fs::read_to_string("static/verify.html")
                    .await
                    .map_err(|err| {
                        log::error!("Couldn't read verify preset: {}", err);
                        error!(SERVER, "Could not send email")
                    })?
                    .replace("${CODE}", &code);
                (email.subjects.verify.replace("${CODE}", &code), content)
            }
        };
        let mut message = Message::builder()
            .from(
                format!("{} <{}>", email.name, email.address)
                    .parse()
                    .map_err(|err| {
                        log::error!("Failed to build email message: {}", err);
                        error!(SERVER, "Could not send email")
                    })?,
            )
            .to(to.parse().map_err(|err| {
                log::error!("Failed to build email message: {}", err);
                error!(SERVER, "Could not send email")
            })?);
        if !subject.is_empty() {
            message = message.subject(subject);
        }
        let message = message
            .singlepart(SinglePart::html(content))
            .map_err(|err| {
                log::error!("Failed to build email message: {}", err);
                error!(SERVER, "Could not send verification email")
            })?;
        self.0
            .as_ref()
            .unwrap()
            .send(message)
            .await
            .map_err(|err| {
                log::error!("Failed to send email: {}", err);
                error!(SERVER, "Could not send verification email")
            })?;
        Ok(())
    }
}
