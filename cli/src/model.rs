use crate::youtube::{ChannelResponseItem, Response};
use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct Category {
    pub id: String,
    pub title: String,
    pub total_subscribers: i32,
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize, Clone, Default)]
#[serde(default)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub category: String,
    pub title: String,
    pub description: String,
    pub thumbnail: String,
    pub link: String,
    pub subscriber_count: i32,
    pub video_count: i32,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct CategoryListItem {
    pub channels: Vec<Channel>,
    pub total_subscribers: i32,
}

impl TryFrom<Response<ChannelResponseItem>> for Channel {
    type Error = ();

    fn try_from(res: Response<ChannelResponseItem>) -> Result<Self, Self::Error> {
        let mut ch = Channel::default();

        let statistics = &res.items[0].statistics;
        let channel_snippet = &res.items[0].snippet;
        let subscriber_count = statistics.subscriber_count.parse::<i32>().unwrap();
        let video_count = statistics.video_count.parse::<i32>().unwrap();

        // skip if number of subscribers is not enough
        // skip if number of videos is not enough
        if subscriber_count < 10 || video_count < 5 {
            return Err(());
        }

        ch.id = res.items[0].id.clone();
        ch.title = channel_snippet.title.clone();
        ch.description = channel_snippet.description.clone();
        ch.thumbnail = channel_snippet.thumbnails.default.url.clone();
        ch.link = format!("https://www.youtube.com/channel/{}", ch.id);
        ch.subscriber_count = subscriber_count;
        ch.video_count = video_count;
        ch.created_at = Some(channel_snippet.published_at);

        Ok(ch)
    }
}
