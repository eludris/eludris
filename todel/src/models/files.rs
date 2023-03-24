use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

/// The data Effis provides for files
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileData {
    #[serde_as(as = "DisplayFromStr")]
    pub id: u128,
    pub name: String,
    pub bucket: String,
    #[serde(default = "spoiler_default")]
    #[serde(skip_serializing_if = "is_false")]
    pub spoiler: bool,
    pub metadata: FileMetadata,
}

fn is_false(value: &bool) -> bool {
    !value
}

fn spoiler_default() -> bool {
    false
}

/// The enum representing all the possible Effis supported file metadatas
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum FileMetadata {
    Text,
    Image {
        #[serde(skip_serializing_if = "Option::is_none")]
        width: Option<usize>,
        #[serde(skip_serializing_if = "Option::is_none")]
        height: Option<usize>,
    },
    Video {
        #[serde(skip_serializing_if = "Option::is_none")]
        width: Option<usize>,
        #[serde(skip_serializing_if = "Option::is_none")]
        height: Option<usize>,
    },
    Other,
}

#[cfg(feature = "logic")]
pub struct File {
    pub id: u128,
    pub file_id: u128,
    pub name: String,
    pub content_type: String,
    pub hash: String,
    pub bucket: String,
    pub spoiler: bool,
    pub width: Option<usize>,
    pub height: Option<usize>,
}

#[cfg(feature = "http")]
pub use file_logic::*;

#[cfg(feature = "http")]
mod file_logic {
    #![allow(clippy::unnecessary_lazy_evaluations)] // Needed because rocket
    use std::path::PathBuf;

    use image::io::Reader as ImageReader;
    use rocket::{
        fs::TempFile,
        http::{ContentType, Header},
        FromForm, Responder,
    };
    use sqlx::{pool::PoolConnection, MySql};
    use tokio::{fs, sync::Mutex};

    use super::File;
    use crate::ids::IDGenerator;
    use crate::models::{
        ErrorResponse, ErrorResponseData, FileData, FileMetadata, NotFoundError, ServerError,
        ValidationError,
    };

    #[derive(Debug, Responder)]
    pub struct FetchResponse<'a> {
        pub file: fs::File,
        pub disposition: Header<'a>,
        pub content_type: ContentType,
    }

    #[derive(Debug, FromForm)]
    pub struct FileUpload<'a> {
        pub file: TempFile<'a>,
        pub spoiler: bool,
    }

    impl File {
        pub async fn create<'a>(
            mut file: TempFile<'a>,
            bucket: String,
            gen: &Mutex<IDGenerator>,
            db: &mut PoolConnection<MySql>,
            spoiler: bool,
        ) -> Result<FileData, ErrorResponse> {
            let id = gen.lock().await.generate_id();
            let path = PathBuf::from(format!("files/{}/{}", bucket, id));
            let name = match file.raw_name() {
                Some(name) => PathBuf::from(name.dangerous_unsafe_unsanitized_raw().as_str())
                    .file_name()
                    .map(|n| n.to_str().unwrap_or("attachment"))
                    .unwrap_or("attachment")
                    .to_string(),
                None => "attachment".to_string(),
            };
            file.persist_to(&path).await.unwrap();
            let data = fs::read(&path).await.unwrap();

            let hash = sha256::digest(&data[..]);
            let file = if let Ok((file_id, content_type, width, height)) = sqlx::query!(
                "
SELECT file_id, content_type, width, height
FROM files
WHERE hash = ?
AND bucket = ?
                ",
                hash,
                bucket,
            )
            .fetch_one(&mut *db)
            .await
            .map(|f| (f.file_id, f.content_type, f.width, f.height))
            {
                fs::remove_file(path).await.unwrap();
                sqlx::query!(
                    "
INSERT INTO files(id, file_id, name, content_type, hash, bucket, spoiler, width, height)
VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?)
                    ",
                    id.to_string(),
                    file_id,
                    name,
                    content_type,
                    hash,
                    bucket,
                    spoiler,
                    width,
                    height,
                )
                .execute(&mut *db)
                .await
                .unwrap();

                Self {
                    id,
                    file_id: file_id.parse().unwrap(),
                    name,
                    content_type,
                    hash,
                    bucket,
                    spoiler,
                    width: width.map(|s| s as usize),
                    height: height.map(|s| s as usize),
                }
            } else {
                let file = tokio::task::spawn_blocking(move || {
                    let mime = tree_magic::from_u8(&data);
                    let (width, height) = match mime.as_ref() {
                        "image/gif" | "image/jpeg" | "image/png" | "image/webp" => {
                            if mime == "image/jepg" {
                                ImageReader::open(&path)
                                    .map_err(|e| {
                                        log::error!(
                                            "Failed to strip file metadata on {} with id {}: {:?}",
                                            name,
                                            id,
                                            e
                                        );
                                        ServerError {
                                            error: "Failed to strip file metadata".to_string(),
                                        }
                                        .to_error_response()
                                    })?
                                    .decode()
                                    .map_err(|e| {
                                        log::error!(
                                            "Failed to strip file metadata on {} with id {}: {:?}",
                                            name,
                                            id,
                                            e
                                        );
                                        ServerError {
                                            error: "Failed to strip file metadata".to_string(),
                                        }
                                        .to_error_response()
                                    })?
                                    .save(&path)
                                    .map_err(|e| {
                                        log::error!(
                                            "Failed to strip image metadata on {} with id {}: {:?}",
                                            name,
                                            id,
                                            e
                                        );
                                        ServerError {
                                            error: "Failed to strip file metadata".to_string(),
                                        }
                                        .to_error_response()
                                    })?;
                            }
                            imagesize::blob_size(&data)
                                .map(|d| (Some(d.width), Some(d.height)))
                                .unwrap_or((None, None))
                        }
                        "video/mp4" | "video/webm" | "video/quicktime" => {
                            if &bucket != "attachments" {
                                std::fs::remove_file(path).unwrap();
                                return Err(ValidationError {
                                    field_name: "content_type".to_string(),
                                    error: "Non atatchment buckets can only have images and gifs"
                                        .to_string(),
                                }
                                .to_error_response());
                            };

                            let mut dimensions = (None, None);
                            for stream in ffprobe::ffprobe(&path)
                                .map_err(|e| {
                                    log::error!(
                                        "Failed to strip video metadata on {} with id {}: {:?}",
                                        name,
                                        id,
                                        e
                                    );
                                    ServerError {
                                        error: "Failed to strip file metadata".to_string(),
                                    }
                                    .to_error_response()
                                })?
                                .streams
                                .iter()
                            {
                                if let (Some(width), Some(height)) = (stream.width, stream.height) {
                                    dimensions = (Some(width as usize), Some(height as usize));
                                    break;
                                }
                            }
                            dimensions
                        }
                        _ => {
                            if &bucket != "attachments" {
                                std::fs::remove_file(path).unwrap();
                                return Err(ValidationError {
                                    field_name: "content_type".to_string(),
                                    error: "Non atatchment buckets can only have images and gifs"
                                        .to_string(),
                                }
                                .to_error_response());
                            };

                            (None, None)
                        }
                    };
                    Ok(Self {
                        id,
                        file_id: id,
                        name,
                        content_type: mime,
                        hash,
                        bucket,
                        spoiler,
                        width,
                        height,
                    })
                })
                .await
                .unwrap()?;
                sqlx::query!(
                    "
INSERT INTO files(id, file_id, name, content_type, hash, bucket, spoiler, width, height)
VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?)
                    ",
                    file.id.to_string(),
                    file.id.to_string(),
                    file.name,
                    file.content_type,
                    file.hash,
                    file.bucket,
                    file.spoiler,
                    file.width.map(|s| s as u32),
                    file.height.map(|s| s as u32),
                )
                .execute(&mut *db)
                .await
                .unwrap();

                file
            };

            Ok(file.get_file_data())
        }

        async fn get<'a>(
            id: u128,
            bucket: &'a str,
            db: &mut PoolConnection<MySql>,
        ) -> Option<Self> {
            sqlx::query!(
                "
SELECT *
FROM files
WHERE id = ?
AND bucket = ?
                ",
                id.to_string(),
                bucket,
            )
            .fetch_one(&mut *db)
            .await
            .map(|r| Self {
                id: r.id.parse().unwrap(),
                file_id: r.file_id.parse().unwrap(),
                name: r.name,
                content_type: r.content_type,
                hash: r.hash,
                bucket: r.bucket,
                spoiler: r.spoiler == 1,
                width: r.width.map(|s| s as usize),
                height: r.height.map(|s| s as usize),
            })
            .ok()
        }

        pub async fn fetch_file<'a>(
            id: u128,
            bucket: &'a str,
            db: &mut PoolConnection<MySql>,
        ) -> Result<FetchResponse<'a>, ErrorResponse> {
            let file_data = Self::get(id, bucket, db)
                .await
                .ok_or_else(|| NotFoundError.to_error_response())?;
            let file = fs::File::open(format!("files/{}/{}", bucket, file_data.file_id))
                .await
                .map_err(|e| {
                    log::error!(
                        "Could not fetch file {} with id {}: {:?}",
                        file_data.name,
                        file_data.id,
                        e
                    );
                    ServerError {
                        error: "Error fetching file".to_string(),
                    }
                    .to_error_response()
                })?;
            Ok(FetchResponse {
                file,
                disposition: Header::new(
                    "Content-Disposition",
                    format!("inline; filename=\"{}\"", file_data.name),
                ),
                content_type: ContentType::parse_flexible(&file_data.content_type).unwrap(),
            })
        }

        pub async fn fetch_file_download<'a>(
            id: u128,
            bucket: &'a str,
            db: &mut PoolConnection<MySql>,
        ) -> Result<FetchResponse<'a>, ErrorResponse> {
            let file_data = Self::get(id, bucket, db)
                .await
                .ok_or_else(|| NotFoundError.to_error_response())?;
            let file = fs::File::open(format!("files/{}/{}", bucket, file_data.file_id))
                .await
                .map_err(|e| {
                    log::error!(
                        "Could not fetch file {} with id {}: {:?}",
                        file_data.name,
                        file_data.id,
                        e
                    );
                    ServerError {
                        error: "Error fetching file".to_string(),
                    }
                    .to_error_response()
                })?;
            Ok(FetchResponse {
                file,
                disposition: Header::new(
                    "Content-Disposition",
                    format!("attachment; filename=\"{}\"", file_data.name),
                ),
                content_type: ContentType::parse_flexible(&file_data.content_type).unwrap(),
            })
        }

        pub async fn fetch_file_data<'a>(
            id: u128,
            bucket: &'a str,
            db: &mut PoolConnection<MySql>,
        ) -> Result<FileData, ErrorResponse> {
            Self::get(id, bucket, db)
                .await
                .ok_or_else(|| NotFoundError.to_error_response())
                .map(|f| f.get_file_data())
        }

        fn get_file_data(self) -> FileData {
            let metadata = match self.content_type.as_ref() {
                "image/gif" | "image/jpeg" | "image/png" | "image/webp" => {
                    if self.width.is_some() && self.height.is_some() {
                        FileMetadata::Image {
                            width: self.width,
                            height: self.height,
                        }
                    } else {
                        FileMetadata::Other
                    }
                }
                "video/mp4" | "video/webm" | "video/quicktime" => {
                    if self.width.is_some() && self.height.is_some() {
                        FileMetadata::Video {
                            width: self.width,
                            height: self.height,
                        }
                    } else {
                        FileMetadata::Other
                    }
                }
                _ if self.content_type.starts_with("text") => FileMetadata::Text,
                _ => FileMetadata::Other,
            };

            FileData {
                id: self.id,
                name: self.name,
                bucket: self.bucket,
                metadata,
                spoiler: self.spoiler,
            }
        }
    }
}
