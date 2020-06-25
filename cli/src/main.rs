use chrono::{DateTime, Duration, Utc};
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

#[derive(Deserialize, Debug)]
struct Category {
    id: String,
    title: String,
    #[serde(default)]
    total_subscribers: i32,
}

#[derive(Deserialize, Debug)]
struct Channel {
    id: String,
    name: String,
    category: String,
}

#[derive(Debug)]
struct CategoryListItem {
    channels: Vec<YoutubeChannel>,
    total_subscribers: i32,
}

// Structs from youtube
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct YoutubeChannel {
    id: String,
    title: String,
    description: String,
    thumbnail: String,
    link: String,
    subscriber_count: i32,
    video_count: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Deserialize, Debug)]
struct YoutubeResponse<T> {
    items: Vec<T>,
}

#[derive(Deserialize, Debug)]
struct YoutubeChannelResponseItem {
    id: String,
    snippet: YoutubeChannelResponseItemSnippet,
    statistics: YoutubeChannelResponseItemStatistics,
}

#[derive(Deserialize, Debug)]
struct YoutubeChannelResponseItemSnippet {
    title: String,
    description: String,
    thumbnails: YoutubeChannelResponseItemThumbnail,
    #[serde(rename = "publishedAt")]
    published_at: DateTime<Utc>,
}

#[derive(Deserialize, Debug)]
struct YoutubeChannelResponseItemThumbnail {
    default: YoutubeChannelResponseItemThumbnailItem,
}

#[derive(Deserialize, Debug)]
struct YoutubeChannelResponseItemThumbnailItem {
    url: String,
}

#[derive(Deserialize, Debug)]
struct YoutubeChannelResponseItemStatistics {
    #[serde(rename = "subscriberCount")]
    subscriber_count: String,
    #[serde(rename = "videoCount")]
    video_count: String,
    #[serde(rename = "viewCount")]
    view_count: String,
}

#[derive(Deserialize, Debug)]
struct YoutubeActivitiesResponseItem {
    id: String,
    snippet: YoutubeActivitiesResponseItemSnippet,
}

#[derive(Deserialize, Debug)]
struct YoutubeActivitiesResponseItemSnippet {
    #[serde(rename = "publishedAt")]
    published_at: DateTime<Utc>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let mut categories = read_categories();
    let channels = read_channels();

    let url_prefix = "https://www.googleapis.com/youtube/v3";
    let key = std::env::var("API_KEY").expect("API_KEY env var is not defined");
    let mut category_list: HashMap<String, CategoryListItem> = HashMap::new();

    for ch in channels {
        let url = format!(
            "{}/channels?part=snippet,statistics&id={}&key={}",
            url_prefix, ch.id, key
        );

        let youtube_channel_res: YoutubeResponse<YoutubeChannelResponseItem> = reqwest::get(&url)
            .await?
            .json::<YoutubeResponse<YoutubeChannelResponseItem>>()
            .await?;

        if youtube_channel_res.items.len() == 0 {
            continue;
        }

        let statistics = &youtube_channel_res.items[0].statistics;
        let channel_snippet = &youtube_channel_res.items[0].snippet;
        let subscriber_count = statistics.subscriber_count.parse::<i32>().unwrap();
        let video_count = statistics.video_count.parse::<i32>().unwrap();

        // skip if number of subscribers is not enough
        // skip if number of videos is not enough
        if subscriber_count < 10 || video_count < 5 {
            continue;
        }

        let url = format!(
            "{}/activities?part=snippet&channelId={}&maxResults=1&key={}",
            url_prefix, ch.id, key
        );

        let youtube_activity_res: YoutubeResponse<YoutubeActivitiesResponseItem> =
            reqwest::get(&url)
                .await?
                .json::<YoutubeResponse<YoutubeActivitiesResponseItem>>()
                .await?;

        if youtube_activity_res.items.len() == 0 {
            continue;
        }

        let activity_snippet = &youtube_activity_res.items[0].snippet;

        let youtube_channel = YoutubeChannel {
            id: ch.id.clone(),
            title: channel_snippet.title.clone(),
            description: channel_snippet.description.clone(),
            thumbnail: channel_snippet.thumbnails.default.url.clone(),
            link: format!("https://www.youtube.com/channel/{}", ch.id),
            subscriber_count,
            video_count,
            created_at: channel_snippet.published_at,
            updated_at: activity_snippet.published_at,
        };

        if category_list.contains_key(&ch.category) {
            let item = category_list.get_mut(&ch.category).unwrap();
            item.total_subscribers += youtube_channel.subscriber_count.clone();
            item.channels.push(youtube_channel);
        } else {
            category_list.insert(
                ch.category,
                CategoryListItem {
                    channels: vec![youtube_channel],
                    total_subscribers: subscriber_count,
                },
            );
        }
    }

    // sort categories
    for c in &mut categories {
        if category_list.contains_key(&c.id) {
            let channels = category_list.get(&c.id).unwrap();
            c.total_subscribers = channels.total_subscribers;
        }
    }
    categories.sort_by(|a, b| b.total_subscribers.cmp(&a.total_subscribers));

    // generate README
    let mut toc = String::new();
    let mut list = String::new();
    for c in categories {
        if category_list.contains_key(&c.id) {
            let mut key = c.title.replace(" ", "-");
            key = key.replace("‌", "");

            toc += &format!("- [{}](#{})\n", c.title, key);
            list += &format!("## {}\n<table><tbody>", c.title);

            let channels = category_list.get_mut(&c.id).unwrap();
            channels
                .channels
                .sort_by(|a, b| b.subscriber_count.cmp(&a.subscriber_count));
            for ch in &channels.channels {
                list += &format!(
                    "<tr><td style=\"text-align: center; padding: 5px;\">\
<img src=\"{}\" alt=\"{}\"/><br/><span title=\"تعداد اعضا\">:thumbsup:<span> {}<br/>\
<span title=\"تعداد ویدیو\">:arrow_forward:<span> {}<br/>\
<span title=\"آخرین فعالیت\">:{}:<span> {}</td>\
<td style=\"vertical-align: top; padding: 5px;\"><a href=\"{}\">:link: <b>{}</b></a><br/>{}</td></tr>",
                    ch.thumbnail,
                    ch.title,
                    ch.subscriber_count,
                    ch.video_count,
                    if ch.updated_at.gt(&Utc::now().checked_sub_signed(Duration::days(6*30)).unwrap()) { "blush" } else { "unamused" },
                    ch.updated_at.date().format("%Y-%m-%d"),
                    ch.link,
                    ch.title,
                    ch.description.trim(),
                );
            }

            list += "</tbody></table>\n\n";
        }
    }

    // return Ok(());

    let mut readme = read_readme_template();
    readme = readme.replace("{TOC}", &toc);
    readme = readme.replace("{LIST}", &list);

    println!("{}", readme);
    Ok(())
}

fn read_readme_template() -> String {
    let mut file = File::open("README.template").expect("Expect to see README.template here!");
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)
        .expect("Cannot read README.template contents");
    buffer
}

fn read_categories() -> Vec<Category> {
    let mut file = File::open("categories.json5").expect("Expect to see categories.json5 here!");
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)
        .expect("Cannot read categories.json5 contents");

    json5::from_str(&buffer).expect("Content of categories.json5 is not a valid json5")
}

fn read_channels() -> Vec<Channel> {
    let mut file = File::open("channels.json5").expect("Expect to see channels.json5 here!");
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)
        .expect("Cannot read channels.json5 contents");

    json5::from_str(&buffer).expect("Content of channels.json5 is not a valid json5")
}
