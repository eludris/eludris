use sqlx::{pool::PoolConnection, Postgres};

use crate::models::{Attachment, AttachmentCreate, ErrorResponse, File};

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

impl Attachment {
    pub async fn create(
        mut attachment: AttachmentCreate,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<Self, ErrorResponse> {
        attachment.validate()?;

        let file = match File::get(attachment.file_id, "attachments", db).await {
            Some(file) => file,
            None => {
                return Err(error!(
                    VALIDATION,
                    "file_id", "Attachment file doesn't exist"
                ))
            }
        };

        return Ok(Self {
            file: file.get_file_data(),
            description: attachment.description,
            spoiler: attachment.spoiler,
        });
    }
}
