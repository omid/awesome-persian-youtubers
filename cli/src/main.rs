use std::fs::File;
use std::io::Read;
use serde_derive::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
struct Category {
    id: String,
    title: String,
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
}

#[derive(Deserialize, Debug)]
struct YoutubeResponse {
    items: Vec<YoutubeResponseItem>,
}

#[derive(Deserialize, Debug)]
struct YoutubeResponseItem {
    id: String,
    snippet: YoutubeResponseItemSnippet,
    statistics: YoutubeResponseItemStatistics,
}

#[derive(Deserialize, Debug)]
struct YoutubeResponseItemSnippet {
    title: String,
    description: String,
    thumbnails: YoutubeResponseItemThumbnail,
}

#[derive(Deserialize, Debug)]
struct YoutubeResponseItemThumbnail {
    default: YoutubeResponseItemThumbnailItem,
}

#[derive(Deserialize, Debug)]
struct YoutubeResponseItemThumbnailItem {
    url: String,
}

#[derive(Deserialize, Debug)]
struct YoutubeResponseItemStatistics {
    #[serde(rename = "subscriberCount")]
    subscriber_count: String,
    #[serde(rename = "videoCount")]
    video_count: String,
    #[serde(rename = "viewCount")]
    view_count: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let categories = read_categories();
    let channels = read_channels();

    let domain = "https://www.googleapis.com/youtube/v3/channels";
    let key = std::env::var("API_KEY").expect("API_KEY env var is not defined");
    let mut category_list: HashMap<String, CategoryListItem> = HashMap::new();

    for ch in channels {
        let url = format!("{}?part=snippet,statistics&id={}&key={}", domain, ch.id, key);

        let youtube_res: YoutubeResponse = reqwest::get(&url)
            .await?
            .json::<YoutubeResponse>()
            .await?;

        let youtube_channel = YoutubeChannel {
            id: ch.id.clone(),
            title: youtube_res.items[0].snippet.title.clone(),
            description: youtube_res.items[0].snippet.description.clone(),
            thumbnail: youtube_res.items[0].snippet.thumbnails.default.url.clone(),
            link: format!("https://www.youtube.com/channel/{}", ch.id),
            subscriber_count: youtube_res.items[0].statistics.subscriber_count.parse::<i32>().unwrap(),
            video_count: youtube_res.items[0].statistics.video_count.parse::<i32>().unwrap(),
        };

        // skip if number of subscribers is not enough
        if youtube_channel.subscriber_count < 25 {
            continue;
        }

        // skip if number of videos is not enough
        if youtube_channel.video_count < 5 {
            continue;
        }

        if category_list.contains_key(&ch.category) {
            let item = category_list.get_mut(&ch.category).unwrap();
            item.total_subscribers += youtube_channel.subscriber_count.clone();
            item.channels.push(youtube_channel);
        } else {
            category_list.insert(
                ch.category,
                CategoryListItem {
                    channels: vec![youtube_channel],
                    total_subscribers: youtube_res.items[0].statistics.subscriber_count.parse::<i32>().unwrap()
                }
            );
        }
    }

    let mut toc = String::new();
    let mut list = String::new();
    for c in categories {
        if category_list.contains_key(&c.id) {

            let mut key = c.title.replace(" ", "-");
            key = key.replace("‌", "");

            toc += &format!("- [{}](#{})\n", c.title, key);

            list += &format!("## {}\n", c.title);

            let channels = category_list.get_mut(&c.id).unwrap();
            channels.channels.sort_by(|a, b| b.subscriber_count.cmp(&a.subscriber_count));
            for ch in &channels.channels {
                list += &format!(
                    "* <img src=\"{}\" alt=\"{}\" width=\"25\"/> **{}** <a href=\"{}\">:link:</a> ({} عضو)\n
    {}\n",
                    ch.thumbnail,
                    ch.title,
                    ch.title,
                    ch.link,
                    ch.subscriber_count,
                    ch.description.lines().next().unwrap_or_default(),
                );
            }
        }
    }

    let mut readme = read_readme_template();
    readme = readme.replace("{TOC}", &toc);
    readme = readme.replace("{LIST}", &list);

    println!("{}", readme);
    Ok(())
}

fn read_readme_template() -> String {
    let mut file = File::open("README.template").expect("Expect to see README.template here!");
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).expect("Cannot read README.template contents");
    buffer
}

fn read_categories() -> Vec<Category> {
    let mut file = File::open("categories.json5").expect("Expect to see categories.json5 here!");
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).expect("Cannot read categories.json5 contents");

    json5::from_str(&buffer).expect("Content of categories.json5 is not a valid json5")
}

fn read_channels() -> Vec<Channel> {
    let mut file = File::open("channels.json5").expect("Expect to see channels.json5 here!");
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).expect("Cannot read channels.json5 contents");

    json5::from_str(&buffer).expect("Content of channels.json5 is not a valid json5")
}
