use lettre::{transport::smtp::authentication::Credentials, AsyncSmtpTransport, Tokio1Executor};
use rocket::{
    fairing::{Fairing, Info, Kind, Result},
    Build, Rocket,
};
use todel::{models::Emailer, Conf};

pub struct EmailFairing;

#[rocket::async_trait]
impl Fairing for EmailFairing {
    fn info(&self) -> Info {
        Info {
            name: "Handle managing an SmtpTransport for mailing purposes",
            kind: Kind::Ignite,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> Result {
        let conf = rocket
            .state::<Conf>()
            .expect("Failed to obtain the managed Conf");
        if let Some(email) = &conf.email {
            let mut transport_builder = AsyncSmtpTransport::<Tokio1Executor>::relay(&email.relay)
                .expect("Couldn't initialise an SMTP transport builder");
            if let Some(credentials) = &email.credentials {
                transport_builder = transport_builder.credentials(Credentials::new(
                    credentials.username.clone(),
                    credentials.password.clone(),
                ));
            }
            let mailer = transport_builder.build::<Tokio1Executor>();

            Ok(rocket.manage(Emailer(Some(mailer))))
        } else {
            Ok(rocket.manage(Emailer(None)))
        }
    }
}
