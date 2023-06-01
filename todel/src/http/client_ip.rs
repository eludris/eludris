use std::{convert::Infallible, fmt::Display, net::IpAddr, ops::Deref, str::FromStr};

use rocket::{
    async_trait,
    request::{FromRequest, Outcome, Request},
};

/// The *real* IP of a client.
#[derive(Debug, Clone)]
pub struct ClientIP(IpAddr);

impl Display for ClientIP {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for ClientIP {
    type Target = IpAddr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait]
impl<'r> FromRequest<'r> for ClientIP {
    type Error = Infallible;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // Hey there future reader or probably oliver, in case you're wondering why these two lines
        // got removed it's because apparently rocket already checks the `X-Real-IP` header when
        // the `client_ip` method is called
        //
        // Docs: https://api.rocket.rs/v0.5-rc/rocket/request/struct.Request.html#method.client_ip
        //
        // if let Some(ip) = req.headers().get_one("X-Real-IP") {
        // Outcome::Success(ClientIP(IpAddr::from_str(ip).unwrap()))
        // } else
        if let Some(ip) = req.headers().get_one("CF-Connecting-IP") {
            Outcome::Success(ClientIP(IpAddr::from_str(ip).unwrap()))
        } else {
            Outcome::Success(ClientIP(
                req.client_ip()
                    .unwrap_or_else(|| IpAddr::from_str("127.0.0.1").unwrap()),
            ))
        }
    }
}
