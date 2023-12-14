use lettre::{message::SinglePart, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use tokio::fs;

use crate::{conf::Email, models::ErrorResponse};

pub struct Emailer(pub Option<AsyncSmtpTransport<Tokio1Executor>>);

#[derive(Debug, Clone, PartialEq)]
pub enum EmailPreset<'a> {
    Verify {
        username: &'a str,
        code: u32,
    },
    Delete {
        username: &'a str,
    },
    PasswordReset {
        username: &'a str,
        code: u32,
    },
    UserUpdated {
        username: &'a str,
        old_username: Option<&'a str>,
        new_email: Option<&'a str>,
        password: bool,
    },
}

impl<'a> EmailPreset<'a> {
    pub async fn to_message(self, to: &str, email: &Email) -> Result<Message, ErrorResponse> {
        let (subject, content) = match self {
            EmailPreset::Verify { username, code } => {
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
                    })?;
                let replace =
                    |s: &str| s.replace("${USERNAME}", username).replace("${CODE}", &code);
                (replace(&email.subjects.verify), replace(&content))
            }
            EmailPreset::Delete { username } => {
                let content = fs::read_to_string("static/delete.html")
                    .await
                    .map_err(|err| {
                        log::error!("Couldn't read delete preset: {}", err);
                        error!(SERVER, "Could not send email")
                    })?;
                let replace = |s: &str| s.replace("${USERNAME}", username);
                (replace(&email.subjects.delete), replace(&content))
            }
            EmailPreset::PasswordReset { username, code } => {
                let code = code
                    .to_string()
                    .chars()
                    .collect::<Vec<char>>()
                    .chunks(3)
                    .map(|c| c.iter().collect::<String>())
                    .collect::<Vec<String>>()
                    .join(" ");
                let content = fs::read_to_string("static/password-reset.html")
                    .await
                    .map_err(|err| {
                        log::error!("Couldn't read password-reset preset: {}", err);
                        error!(SERVER, "Could not send email")
                    })?;
                let replace =
                    |s: &str| s.replace("${USERNAME}", username).replace("${CODE}", &code);
                (replace(&email.subjects.password_reset), replace(&content))
            }
            EmailPreset::UserUpdated {
                username,
                old_username,
                new_email,
                password,
            } => {
                let mut changes = String::new();
                let content = fs::read_to_string("static/user-updated.html")
                    .await
                    .map_err(|err| {
                        log::error!("Couldn't read user-updated preset: {}", err);
                        error!(SERVER, "Could not send email")
                    })?;
                if let Some(old_username) = old_username {
                    changes.push_str(&format!(
                        "Your username has been changed from {} to {}",
                        old_username, username
                    ));
                }
                if let Some(new_email) = new_email {
                    if !changes.is_empty() {
                        changes.push('\n');
                    }
                    changes.push_str(&format!("Your email has been changed to {}", new_email));
                }
                if password {
                    if !changes.is_empty() {
                        changes.push('\n');
                    }
                    changes.push_str("Your password has been updated");
                }
                let replace = |s: &str| {
                    s.replace("${USERNAME}", username)
                        .replace("${CHANGES}", &changes)
                };
                (replace(&email.subjects.user_updated), replace(&content))
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
        message
            .singlepart(SinglePart::html(content))
            .map_err(|err| {
                log::error!("Failed to build email message: {}", err);
                error!(SERVER, "Could not send verification email")
            })
    }
}

impl Emailer {
    pub async fn send_email<'a>(
        &self,
        to: &str,
        preset: EmailPreset<'a>,
        email: &Email,
    ) -> Result<(), ErrorResponse> {
        self.0
            .as_ref()
            .unwrap()
            .send(preset.to_message(to, email).await?)
            .await
            .map_err(|err| {
                log::error!("Failed to send email: {}", err);
                error!(SERVER, "Could not send verification email")
            })?;
        Ok(())
    }
}
