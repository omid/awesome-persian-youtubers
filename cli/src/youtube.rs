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
pub struct Request<'a> {
    url_prefix: &'a str,
    id: &'a str,
    key: &'a str,
}

struct ChannelRequest;

struct ActivitiesRequest;

impl<'a> Request<'a> {
    pub fn new(id: &'a str, key: &'a str) -> Self {
        Self {
            url_prefix: "https://www.googleapis.com/youtube/v3",
            id,
            key,
        }
    }

    pub async fn get_channel(&self) -> Response<ChannelResponseItem> {
        self.get(ChannelRequest)
            .await
            .json::<Response<ChannelResponseItem>>()
            .await
            .unwrap()
    }

    pub async fn get_activities(&self) -> Response<ActivitiesResponseItem> {
        self.get(ActivitiesRequest)
            .await
            .json::<Response<ActivitiesResponseItem>>()
            .await
            .unwrap()
    }

    async fn get(&self, request: impl RequestTrait) -> reqwest::Response {
        reqwest::get(&request.get_url(self)).await.unwrap()
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
