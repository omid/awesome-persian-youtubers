use crate::youtube::Request;
use chrono::{DateTime, Datelike, Duration, Utc};
use persian::english_to_persian_digits;
use ptime::from_gregorian_date;
use serde::de;
use serde_derive::Deserialize;
use std::collections::HashMap;
use tokio::fs;

#[cfg(test)]
mod tests;
mod youtube;

#[derive(Deserialize, Debug)]
struct Category {
    id: String,
    title: String,
    #[serde(default)]
    total_subscribers: i32,
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize)]
struct Channel {
    id: String,
    name: String,
    category: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    thumbnail: String,
    #[serde(default)]
    link: String,
    #[serde(default)]
    subscriber_count: i32,
    #[serde(default)]
    video_count: i32,
    #[serde(default)]
    created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
struct CategoryListItem {
    channels: Vec<Channel>,
    total_subscribers: i32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let mut categories: Vec<Category> = read_json_file("categories.json5").await?;
    let channels: Vec<Channel> = read_json_file("channels.json5").await?;

    let key = std::env::var("API_KEY").expect("API_KEY env var is not defined");
    let mut category_list: HashMap<String, CategoryListItem> = HashMap::new();

    for mut ch in channels {
        let req = Request::new(&ch.id, &key);
        let youtube_channel_res = req.get_channel().await;

        if youtube_channel_res.items.is_empty() {
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

        let youtube_activity_res = req.get_activities().await;

        if youtube_activity_res.items.is_empty() {
            continue;
        }

        let activity_snippet = &youtube_activity_res.items[0].snippet;

        ch.title = channel_snippet.title.clone();
        ch.description = channel_snippet.description.clone();
        ch.thumbnail = channel_snippet.thumbnails.default.url.clone();
        ch.link = format!("https://www.youtube.com/channel/{}", ch.id);
        ch.subscriber_count = subscriber_count;
        ch.video_count = video_count;
        ch.created_at = Some(channel_snippet.published_at);
        ch.updated_at = Some(activity_snippet.published_at);

        if category_list.contains_key(&ch.category) {
            let item = category_list.get_mut(&ch.category).unwrap();
            item.total_subscribers += ch.subscriber_count;
            item.channels.push(ch);
        } else {
            category_list.insert(
                ch.category.clone(),
                CategoryListItem {
                    channels: vec![ch],
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
                let p_date = from_gregorian_date(
                    ch.updated_at.unwrap().date().year(),
                    ch.updated_at.unwrap().date().month() as i32 - 1,
                    ch.updated_at.unwrap().date().day() as i32,
                )
                .unwrap();

                list += &format!(
                    "<tr><td style=\"text-align: center; padding: 5px; vertical-align: top;\">\
<img src=\"{}\" alt=\"{}\"/><br/><span title=\"تعداد اعضا\">:thumbsup:<span> {}<br/>\
<span title=\"تعداد ویدیو\">:arrow_forward:<span> {}<br/>\
<span title=\"آخرین فعالیت\">:{}:<span> {}</td>\
<td style=\"vertical-align: top; padding: 5px;\"><a href=\"{}\">:link: <b>{}</b></a><br/>{}</td></tr>",
                    ch.thumbnail,
                    ch.title,
                    english_to_persian_digits(&ch.subscriber_count.to_string()),
                    english_to_persian_digits(&ch.video_count.to_string()),
                    if ch.updated_at.unwrap().gt(&Utc::now().checked_sub_signed(Duration::days(6*30)).unwrap()) { "blush" } else { "unamused" },
                    english_to_persian_digits(&p_date.to_string("yyyy/MM/dd")),
                    ch.link,
                    ch.title,
                    ch.description.trim(),
                );
            }

            list += "</tbody></table>\n\n";
        }
    }

    let mut readme = read_string_file("README.template").await?;
    readme = readme.replace("{TOC}", &toc);
    readme = readme.replace("{LIST}", &list);

    println!("{}", readme);
    Ok(())
}

async fn read_string_file(file_name: &str) -> Result<String, String> {
    fs::read_to_string(file_name)
        .await
        .map_err(|e| format!("Read {} failed error messsage {}", file_name, e))
}

async fn read_json_file<T>(file_name: &str) -> Result<T, String>
where
    T: de::DeserializeOwned,
{
    let content = read_string_file(file_name).await?;
    json5::from_str::<T>(&*content).map_err(|e| {
        format!(
            "convert json file {} failed error messsage {}",
            file_name, e
        )
    })
}
