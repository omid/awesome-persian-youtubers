use chrono::{DateTime, Utc};
use serde_derive::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Response<T> {
    pub items: Vec<T>,
}

#[derive(Deserialize, Debug)]
pub struct ChannelResponseItem {
    pub id: String,
    pub snippet: ChannelResponseItemSnippet,
    pub statistics: ChannelResponseItemStatistics,
}

#[derive(Deserialize, Debug)]
pub struct ChannelResponseItemSnippet {
    pub title: String,
    pub description: String,
    pub thumbnails: ChannelResponseItemThumbnail,
    #[serde(rename = "publishedAt")]
    pub published_at: DateTime<Utc>,
}

#[derive(Deserialize, Debug)]
pub struct ChannelResponseItemThumbnail {
    pub default: ChannelResponseItemThumbnailItem,
}

#[derive(Deserialize, Debug)]
pub struct ChannelResponseItemThumbnailItem {
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct ChannelResponseItemStatistics {
    #[serde(rename = "subscriberCount")]
    pub subscriber_count: String,
    #[serde(rename = "videoCount")]
    pub video_count: String,
    #[serde(rename = "viewCount")]
    view_count: String,
}

#[derive(Deserialize, Debug)]
pub struct ActivitiesResponseItem {
    pub id: String,
    pub snippet: ActivitiesResponseItemSnippet,
}

#[derive(Deserialize, Debug)]
pub struct ActivitiesResponseItemSnippet {
    #[serde(rename = "publishedAt")]
    pub published_at: DateTime<Utc>,
}

trait RequestTrait {
    fn get_url(&self, request: &Request) -> String;
}

#[derive(Deserialize, Debug)]
pub struct Request {
    url_prefix: String,
    id: String,
    key: String,
}

struct ChannelRequest;

struct ActivitiesRequest;

impl Request {
    pub fn new(id: &str, key: &str) -> Self {
        Self {
            url_prefix: "https://www.googleapis.com/youtube/v3".to_string(),
            id: id.to_string(),
            key: key.to_string(),
        }
    }

    pub async fn get_channel(&self) -> Result<Response<ChannelResponseItem>, String> {
        self.get(ChannelRequest)
            .await?
            .json::<Response<ChannelResponseItem>>()
            .await
            .map_err(|e| {
                format!(
                    "Youtube response is not valid for \"{}\" with \"{}\" error.",
                    ChannelRequest.get_url(self),
                    e
                )
            })
    }

    pub async fn get_activities(&self) -> Result<Response<ActivitiesResponseItem>, String> {
        self.get(ActivitiesRequest)
            .await?
            .json::<Response<ActivitiesResponseItem>>()
            .await
            .map_err(|e| {
                format!(
                    "Youtube response is not valid for \"{}\" with \"{}\" error.",
                    ActivitiesRequest.get_url(self),
                    e
                )
            })
    }

    async fn get(&self, request: impl RequestTrait) -> Result<reqwest::Response, String> {
        reqwest::get(&request.get_url(self)).await.map_err(|e| {
            format!(
                "Youtube response is not valid for \"{}\" with \"{}\" error.",
                request.get_url(self),
                e,
            )
        })
    }
}

impl RequestTrait for ChannelRequest {
    fn get_url(&self, request: &Request) -> String {
        format!(
            "{}/channels?part=snippet,statistics&id={}&key={}",
            request.url_prefix, request.id, request.key
        )
    }
}

impl RequestTrait for ActivitiesRequest {
    fn get_url(&self, request: &Request) -> String {
        format!(
            "{}/activities?part=snippet&channelId={}&maxResults=1&key={}",
            request.url_prefix, request.id, request.key
        )
    }
}
