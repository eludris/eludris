use std::{collections::HashMap, str::FromStr, time::Duration};

use redis::AsyncCommands;
use regex::Regex;
use reqwest::{header::CONTENT_TYPE, redirect::Policy, Client};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use sqlx::{pool::PoolConnection, Postgres};
use tokio::fs;
use url::Url;

use crate::models::{Embed, ErrorResponse, Message, ServerPayload};

impl Message {
    pub async fn populate_embeds<C: AsyncCommands>(
        &self,
        mut db: PoolConnection<Postgres>,
        mut cache: C,
    ) -> Result<(), ErrorResponse> {
        let content = match &self.content {
            Some(content) => content,
            None => return Ok(()),
        };
        lazy_static! {
            static ref CLIENT: Client = Client::builder()
                .timeout(Duration::from_secs(10))
                .redirect(Policy::limited(5))
                .user_agent(
                    concat!(
                        "Mozilla/5.0 (compatible; eludris/", env!("CARGO_PKG_VERSION"), ";)"
                    )
                )
                .build()
                .expect("Couldn't build reqwest client");
            static ref URL_REGEX: Regex = Regex::new(
                r"https?:\/\/(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)"
            ).expect("Failed to compile URL regex");
        };

        let mut embeds: Vec<Embed> = vec![];
        let urls: Vec<Url> = URL_REGEX
            .find_iter(content)
            .filter_map(|m| Url::from_str(m.as_str()).ok())
            .collect();

        for url in urls {
            if let Ok(Some(embed)) = cache
                .get::<_, Option<String>>(format!("embed:{}", url))
                .await
                .map_err(|err| {
                    log::warn!("Failed to fetch embed data from cache for {}: {}", url, err)
                })
            {
                embeds.push(serde_json::from_str(&embed).unwrap());
                continue;
            }
            match generate_website_embed(&url, &CLIENT).await {
                Some(embed) => {
                    cache
                        .set_ex::<_, _, ()>(
                            format!("embed:{}", url),
                            serde_json::to_string(&embed).unwrap(),
                            7200,
                        )
                        .await
                        .map_err(|err| {
                            log::warn!(
                                "Failed to commit embed data from cache for {}: {}",
                                url,
                                err
                            )
                        })
                        .ok();
                    embeds.push(embed);
                }
                None => continue,
            }
        }

        for embed in embeds.iter() {
            sqlx::query!(
                "
                INSERT INTO message_embeds(message_id, embed)
                VALUES($1, $2)
                ",
                self.id as i64,
                serde_json::to_value(embed.clone()).unwrap(),
            )
            .execute(&mut *db)
            .await
            .map_err(|err| {
                log::error!(
                    "Couldn't add message embed {:?} to {}: {}",
                    embed,
                    self.id,
                    err
                );
                error!(SERVER, "Failed to create message")
            })?;
        }

        if !embeds.is_empty() {
            cache
                .publish::<&str, String, ()>(
                    "eludris-events",
                    serde_json::to_string(&ServerPayload::MessageEmbedPopulate {
                        channel_id: self.channel.get_id(),
                        message_id: self.id,
                        embeds,
                    })
                    .unwrap(),
                )
                .await
                .unwrap();
        }

        Ok(())
    }
}

pub async fn generate_website_embed(url: &Url, client: &Client) -> Option<Embed> {
    let resp = client
        .get(url.to_string())
        .send()
        .await
        .map_err(|err| {
            log::debug!(
                "Failed to fetch data at {} for embed: {}",
                url.to_string(),
                err
            );
        })
        .ok()?;
    let (content_type, content_subtype) = resp
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|header_value| header_value.to_str().ok())
        .and_then(|s| s.split_once('/'))
        .map(|(t, st)| (t.to_lowercase(), st.to_lowercase()))?;
    let content_subtype = content_subtype.split(";").next()?;
    if content_subtype == "html" {
        let body = resp.text().await.ok()?;
        let (mut metadata, title, oembed_url) = {
            let document = Html::parse_document(&body);
            let mut metadata = HashMap::new();
            for tag in document.select(&Selector::parse("meta").unwrap()) {
                let value = tag.value();
                if let (Some(property), Some(value)) = (
                    value.attr("property").or_else(|| value.attr("name")),
                    value.attr("content"),
                ) {
                    metadata.insert(property.to_string(), value.to_string());
                }
            }
            for tag in document.select(&Selector::parse("link").unwrap()) {
                let value = tag.value();
                if let (Some(property), Some(value)) = (value.attr("rel"), value.attr("href")) {
                    metadata.insert(property.to_string(), value.to_string());
                }
            }
            let oembed_url = document
                .select(&Selector::parse("link[type=\"application/json+oembed\"]").unwrap())
                .next()
                .and_then(|f| f.attr("href"))
                .map(|u| u.to_string());
            (
                metadata,
                document
                    .select(&Selector::parse("title").unwrap())
                    .next()?
                    .text()
                    .next()
                    .map(|t| t.to_string()),
                oembed_url,
            )
        };
        let domain = url.domain()?.replace("www.", "");
        if domain == "youtube.com" || domain == "youtu.be" {
            if let Some(oembed_url) = oembed_url {
                if let Some(embed) =
                    generate_youtube_video_embed(url, client, &oembed_url, &metadata).await
                {
                    return Some(embed);
                };
            };
        } else if domain == "open.spotify.com" || domain == "spotify.link" {
            if let Some(oembed_url) = oembed_url {
                if let Some(embed) = generate_spotify_embed(url, client, &oembed_url).await {
                    return Some(embed);
                };
            };
        }
        let mut image = None;
        let mut image_width = None;
        let mut image_height = None;
        if let Some(image_url) = metadata
            .remove("og:image")
            .or_else(|| metadata.remove("og:image:secure_url"))
            .or_else(|| metadata.remove("twitter:image"))
            .or_else(|| metadata.remove("twitter:image:src"))
            .map(|s| s.trim().to_owned())
        {
            if let Ok(image_resp) = client.get(&image_url).send().await {
                let bytes = image_resp.bytes().await.ok()?;
                let size = imagesize::blob_size(&bytes).ok()?;
                image = Some(image_url);
                image_width = Some(size.width as u32);
                image_height = Some(size.height as u32);
            }
        }
        let mut description = metadata
            .remove("og:description")
            .or_else(|| metadata.remove("twitter:description"))
            .or_else(|| metadata.remove("description"))
            .map(|s| s.trim().to_owned());
        if let Some(ref mut description) = &mut description {
            if description.len() > 4096 {
                description.truncate(4069);
                description.push_str("...");
            }
        }
        return Some(Embed::Website {
            url: url.to_string(),
            name: metadata.remove("og:site_name").map(|s| s.trim().to_owned()),
            title: metadata
                .remove("og:title")
                .or_else(|| metadata.remove("twitter:title"))
                .or_else(|| metadata.remove("title"))
                .or(title)
                .map(|s| s.trim().to_owned()),
            description,
            colour: metadata.remove("theme-color").map(|s| s.trim().to_owned()),
            image,
            image_width,
            image_height,
        });
    } else if content_type == "image" {
        let bytes = resp.bytes().await.ok()?;
        let size = imagesize::blob_size(&bytes).ok()?;
        return Some(Embed::Image {
            url: url.to_string(),
            width: size.width as u32,
            height: size.height as u32,
        });
    } else if content_type == "video" {
        let bytes = resp.bytes().await.ok()?;
        let path = format!("files/{}", url);
        let path_clone = path.clone();
        fs::write(&path, &bytes).await.ok()?;
        let (width, height) = tokio::task::spawn_blocking(move || {
            let mut dimensions = (None, None);
            let data = match ffprobe::ffprobe(&path).map_err(|err| {
                log::error!("Failed to determine video size for {}: {:?}", path, err);
            }) {
                Ok(data) => data,
                Err(_) => return dimensions,
            };
            for stream in data.streams.iter() {
                if let (Some(width), Some(height)) = (stream.width, stream.height) {
                    dimensions = (Some(width as usize), Some(height as usize));
                    break;
                }
            }
            dimensions
        })
        .await
        .unwrap();
        fs::remove_dir(&path_clone).await.ok()?;
        if let (Some(width), Some(height)) = (width, height) {
            return Some(Embed::Video {
                url: url.to_string(),
                width: width as u32,
                height: height as u32,
            });
        }
    }
    None
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct OEmbed {
    title: Option<String>,
    author_name: Option<String>,
    author_url: Option<String>,
    html: Option<String>,
}

pub async fn generate_youtube_video_embed(
    url: &Url,
    client: &Client,
    oembed_url: &str,
    metadata: &HashMap<String, String>,
) -> Option<Embed> {
    let domain = url.domain()?.replace("www.", "");
    let mut query = HashMap::new();
    for (k, v) in url.query_pairs() {
        query.insert(k.to_string(), v.to_string());
    }
    let id = match domain.as_ref() {
        "youtube.com" => query.get("v")?,
        "youtu.be" => url.path(),
        _ => return None,
    };
    let oembed_data: OEmbed = client
        .get(oembed_url)
        .send()
        .await
        .ok()?
        .json()
        .await
        .ok()?;
    let timestamp = query.get("t").and_then(|t| t.parse().ok());
    Some(Embed::YouTubeVideo {
        url: url.to_string(),
        title: oembed_data.title?,
        video_id: id.to_string(),
        description: metadata
            .get("og:description")
            .or_else(|| metadata.get("twitter:description"))
            .or_else(|| metadata.get("description"))
            .map(|s| s.trim().to_string()),
        channel: oembed_data.author_name?,
        channel_url: oembed_data.author_url?,
        timestamp,
    })
}

pub async fn generate_spotify_embed(url: &Url, client: &Client, oembed_url: &str) -> Option<Embed> {
    let oembed_data: OEmbed = client
        .get(oembed_url)
        .send()
        .await
        .ok()?
        .json()
        .await
        .ok()?;
    Some(Embed::Spotify {
        url: url.to_string(),
        title: oembed_data.title?,
        iframe: oembed_data.html?,
    })
}
