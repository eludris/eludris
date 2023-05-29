use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SessionTokenClaims {
    user_id: u64,
    session_id: u64,
}

impl SessionCreate {
    pub fn ensure_valid(&mut self) {
        self.platform = self.platform.to_lowercase();
        self.client = self.client.to_lowercase();
    }
}
