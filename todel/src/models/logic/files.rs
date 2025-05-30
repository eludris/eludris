#[cfg(feature = "http")]
use std::path::PathBuf;

#[cfg(feature = "http")]
use image::{imageops::FilterType, io::Reader as ImageReader, ImageFormat};
#[cfg(feature = "http")]
use rocket::{
    fs::TempFile,
    http::{ContentType, Header},
    FromForm, Responder,
};
use sqlx::{pool::PoolConnection, Postgres};
#[cfg(feature = "http")]
use tokio::fs;

#[cfg(feature = "http")]
use crate::ids::IdGenerator;
use crate::{
    error,
    models::{ErrorResponse, FileData, FileMetadata},
};

use crate::models::File;

#[cfg(feature = "http")]
#[derive(Debug, Responder)]
pub struct FetchResponse<'a> {
    pub file: fs::File,
    pub disposition: Header<'a>,
    pub content_type: ContentType,
}

#[cfg(feature = "http")]
pub const RESIZABLE_BUCKETS: [&str; 4] = ["avatars", "sphere-icons", "member-avatars", "emojis"];
#[cfg(feature = "http")]
pub const SIZES: [u32; 1] = [256];

/// The data format for uploading a file.
///
/// This is a `multipart/form-data` form.
///
/// -----
///
/// ### Example
///
/// ```sh
/// curl \
///   -F file=@trolley.mp4 \
///   -F spoiler=true \
///   https://cdn.eludris.gay/attachments/
/// ```
#[cfg(feature = "http")]
#[autodoc(category = "Files", hidden = true)]
#[derive(Debug, FromForm)]
pub struct FileUpload<'a> {
    pub file: TempFile<'a>,
    pub spoiler: bool,
}

impl File {
    #[cfg(feature = "http")]
    pub async fn create<'a>(
        mut file: TempFile<'a>,
        bucket: String,
        id_generator: &mut IdGenerator,
        db: &mut PoolConnection<Postgres>,
        spoiler: bool,
    ) -> Result<FileData, ErrorResponse> {
        if file.len() == 0 {
            return Err(error!(
                VALIDATION,
                "file", "You cannot upload an empty file"
            ));
        }

        let id = id_generator.generate();
        let path = PathBuf::from(format!("files/{}/{}", bucket, id));
        let name = match file.raw_name() {
            Some(name) => PathBuf::from(name.dangerous_unsafe_unsanitized_raw().as_str())
                .file_name()
                .map(|n| n.to_str().unwrap_or("attachment"))
                .unwrap_or("attachment")
                .to_string(),
            None => "attachment".to_string(),
        };
        if name.is_empty() || name.len() > 256 {
            return Err(error!(
                VALIDATION,
                "name", "Invalid file name. File name must be between 1 and 256 characters long"
            ));
        }
        file.persist_to(&path).await.unwrap();
        let data = fs::read(&path).await.unwrap();

        let hash = sha256::digest(&data[..]);
        let file = if let Ok((file_id, content_type, width, height)) = sqlx::query!(
            "
SELECT file_id, content_type, width, height
FROM files
WHERE hash = $1
AND bucket = $2
            ",
            hash,
            bucket,
        )
        .fetch_one(&mut **db)
        .await
        .map(|f| (f.file_id, f.content_type, f.width, f.height))
        {
            fs::remove_file(path).await.unwrap();
            sqlx::query!(
                "
INSERT INTO files(id, file_id, name, content_type, hash, bucket, spoiler, width, height)
VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9)
                ",
                id as i64,
                file_id as i64,
                name,
                content_type,
                hash,
                bucket,
                spoiler,
                width as Option<i32>,
                height as Option<i32>,
            )
            .execute(&mut **db)
            .await
            .unwrap();

            Self {
                id,
                file_id: file_id as u64,
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
                let mut mime = tree_magic::from_u8(&data);
                if mime == "application/x-riff" && name.ends_with(".webp") { // tree magic bug
                    mime = "image/webp".to_string();
                }
                let (width, height) = match mime.as_str() {
                    "image/gif" | "image/jpeg" | "image/png" | "image/webp" => {
                        if mime == "image/jpeg" {
                            let mut reader = ImageReader::open(&path)
                                .map_err(|e| {
                                    log::error!(
                                        "Failed to strip file metadata on {} while opening file with id {}: {:?}",
                                        name,
                                        id,
                                        e
                                    );
                                    error!(SERVER, "Failed to strip file metadata")
                                })?;
                            reader.set_format(ImageFormat::Jpeg);
                            reader.decode()
                            .map_err(|e| {
                                log::error!(
                                    "Failed to strip file metadata on {} while decoding with id {}: {:?}",
                                    name,
                                    id,
                                    e
                                );
                                error!(SERVER, "Failed to strip file metadata")
                            })?
                            .save_with_format(&path, ImageFormat::Jpeg)
                            .map_err(|e| {
                                log::error!(
                                    "Failed to strip image metadata on {} while saving with id {}: {:?}",
                                    name,
                                    id,
                                    e
                                );
                                error!(SERVER, "Failed to strip file metadata")
                            })?;
                        }
                        imagesize::blob_size(&data)
                            .map(|d| (Some(d.width), Some(d.height)))
                            .unwrap_or((None, None))
                    }
                    "video/mp4" | "video/webm" | "video/quicktime" => {
                        if &bucket != "attachments" {
                            std::fs::remove_file(path).unwrap();
                            return Err(error!(
                                VALIDATION,
                                "content_type",
                                "Non attachment buckets can only have images and gifs"
                            ));
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
                                error!(SERVER, "Failed to strip file metadata")
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
                            return Err(error!(
                                VALIDATION,
                                "content_type",
                                "Non attachment buckets can only have images and gifs"
                            ));
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
VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9)
                ",
                file.id as i64,
                file.id as i64,
                file.name,
                file.content_type,
                file.hash,
                file.bucket,
                file.spoiler,
                file.width.map(|s| s as i32),
                file.height.map(|s| s as i32),
            )
            .execute(&mut **db)
            .await
            .unwrap();

            file
        };

        Ok(file.get_file_data())
    }

    #[cfg(feature = "http")]
    pub async fn open_file(&self, size: Option<u32>) -> Result<fs::File, ErrorResponse> {
        let mut path = format!("files/{}/{}", self.bucket, self.file_id);
        let data = self.get_file_data();
        if let Some(size) = size {
            if !matches!(data.metadata, FileMetadata::Image { .. }) {
                return Err(error!(VALIDATION, "bucket", "Only images support resizing"));
            } else if !RESIZABLE_BUCKETS.contains(&self.bucket.as_str()) {
                return Err(error!(
                    VALIDATION,
                    "bucket", "This bucket doesn't support resizing"
                ));
            } else if !SIZES.contains(&size) {
                return Err(error!(VALIDATION, "size", "Unsupported size"));
            }
            let old_path = path.clone();
            path = format!("{}-{}", path, size);
            if !fs::try_exists(&path).await.map_err(|e| {
                log::error!(
                    "Could not fetch file {} with id {}: {:?}",
                    self.name,
                    self.id,
                    e
                );
                error!(SERVER, "Error fetching file")
            })? {
                let content_type = self.content_type.clone();
                return Ok(fs::File::from_std(
                    tokio::task::spawn_blocking(move || {
                        let mut reader = ImageReader::open(&old_path).map_err(|e| {
                            log::error!("Failed to open file for resizing at {}: {}", path, e);
                            error!(SERVER, "Failed to resize file")
                        })?;
                        reader.set_format(ImageFormat::from_mime_type(content_type).unwrap());
                        reader
                            .decode()
                            .map_err(|e| {
                                log::error!(
                                    "Failed to strip open file for resizing at {}: {}",
                                    path,
                                    e
                                );
                                error!(SERVER, "Failed to resize file")
                            })?
                            .resize(size, size, FilterType::Nearest)
                            .save_with_format(&path, ImageFormat::Jpeg)
                            .map_err(|e| {
                                log::error!("Failed to write file at {}: {}", path, e);
                                error!(SERVER, "Failed to resize file")
                            })?;
                        std::fs::File::open(&path).map_err(|e| {
                            log::error!("Failed to open file at {}: {}", path, e);
                            error!(SERVER, "Failed to resize file")
                        })
                    })
                    .await
                    .unwrap()?,
                ));
            }
        }
        fs::File::open(path).await.map_err(|e| {
            log::error!(
                "Could not fetch file {} with id {}: {:?}",
                self.name,
                self.id,
                e
            );
            error!(SERVER, "Error fetching file")
        })
    }

    pub async fn get<'a>(
        id: u64,
        bucket: &'a str,
        db: &mut PoolConnection<Postgres>,
    ) -> Option<Self> {
        sqlx::query!(
            "
SELECT *
FROM files
WHERE id = $1
AND bucket = $2
            ",
            id as i64,
            bucket,
        )
        .fetch_one(&mut **db)
        .await
        .map(|r| Self {
            id: r.id as u64,
            file_id: r.file_id as u64,
            name: r.name,
            content_type: r.content_type,
            hash: r.hash,
            bucket: r.bucket,
            spoiler: r.spoiler,
            width: r.width.map(|s| s as usize),
            height: r.height.map(|s| s as usize),
        })
        .ok()
    }

    #[cfg(feature = "http")]
    pub async fn fetch_file<'a>(
        id: u64,
        bucket: &'a str,
        size: Option<u32>,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<FetchResponse<'a>, ErrorResponse> {
        let file_data = Self::get(id, bucket, db)
            .await
            .ok_or_else(|| error!(NOT_FOUND))?;
        let file = file_data.open_file(size).await?;
        Ok(FetchResponse {
            file,
            disposition: Header::new(
                "Content-Disposition",
                format!("inline; filename=\"{}\"", file_data.name),
            ),
            content_type: ContentType::parse_flexible(&file_data.content_type).unwrap(),
        })
    }

    #[cfg(feature = "http")]
    pub async fn fetch_file_download<'a>(
        id: u64,
        bucket: &'a str,
        size: Option<u32>,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<FetchResponse<'a>, ErrorResponse> {
        let file_data = Self::get(id, bucket, db)
            .await
            .ok_or_else(|| error!(NOT_FOUND))?;
        let file = file_data.open_file(size).await?;
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
        id: u64,
        bucket: &'a str,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<FileData, ErrorResponse> {
        Self::get(id, bucket, db)
            .await
            .ok_or_else(|| error!(NOT_FOUND))
            .map(|f| f.get_file_data())
    }

    pub fn get_file_data(&self) -> FileData {
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
            name: self.name.clone(),
            bucket: self.bucket.clone(),
            metadata,
            spoiler: self.spoiler,
        }
    }
}
