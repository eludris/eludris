use crate::models::SessionCreate;

impl SessionCreate {
    pub fn ensure_valid(&mut self) {
        self.platform = self.platform.to_lowercase();
        self.client = self.client.to_lowercase();
    }
}
