use crate::models::{AttachmentCreate, ErrorResponse};

impl AttachmentCreate {
    pub fn validate(&mut self) -> Result<(), ErrorResponse> {
        if let Some(ref mut description) = self.description {
            *description = description.trim().to_string();
            if description.is_empty() {
                self.description = None;
            } else if description.len() > 256 {
                return Err(error!(
                    VALIDATION,
                    "description", "Attachment description has to be less than 256 characters."
                ));
            }
        }
        Ok(())
    }
}
