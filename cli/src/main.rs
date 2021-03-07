use crate::{model::*, youtube::Request};
use chrono::{Datelike, Duration, Utc};
use persian::english_to_persian_digits;
use ptime::from_gregorian_date;
use serde::de;
use std::{collections::HashMap, convert::TryInto, fs::File, io::Write, path::PathBuf};
use structopt::StructOpt;
use tokio::fs;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(short, long, parse(from_os_str))]
    readme: PathBuf,

    #[structopt(short, long, parse(from_os_str))]
    json: PathBuf,

    #[structopt(short, long, parse(from_os_str))]
    yaml: PathBuf,
}

#[macro_use]
mod macros;
mod model;
#[cfg(test)]
mod tests;
mod youtube;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let opt: Opt = Opt::from_args();

    let mut categories: Vec<Category> = read_json_file("categories.json5").await?;
    let raw_channels: Vec<Channel> = read_json_file("channels.json5").await?;

    let key = std::env::var("API_KEY").expect("API_KEY env var is not defined");
    let mut category_list: HashMap<String, CategoryListItem> = HashMap::new();

    for channel in raw_channels {
        let req = Request::new(&channel.id, &key);
        let youtube_channel_res = skip_fail!(req.get_channel().await);

        if youtube_channel_res.items.is_empty() {
            continue;
        }

        let mut ch: Channel = skip_fail!(youtube_channel_res.try_into());
        let youtube_activity_res = skip_fail!(req.get_activities().await);

        if youtube_activity_res.items.is_empty() {
            continue;
        }

        let activity_snippet = &youtube_activity_res.items[0].snippet;
        ch.updated_at = Some(activity_snippet.published_at);
        ch.name = channel.name;
        ch.category = channel.category;

        if category_list.contains_key(&ch.category) {
            let item = category_list.get_mut(&ch.category).unwrap();
            item.total_subscribers += ch.subscriber_count;
            item.channels.push(ch);
        } else {
            category_list.insert(
                ch.category.clone(),
                CategoryListItem {
                    total_subscribers: ch.subscriber_count,
                    channels: vec![ch],
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

    let readme_content = generate_readme(&categories, &mut category_list).await;
    let mut file = File::create(opt.readme)?;
    file.write_all(readme_content.as_bytes())?;

    let category_list = category_list
        .iter()
        .flat_map(|(_, value)| value.channels.clone())
        .collect::<Vec<_>>();

    let json_content = serde_json::to_string(&category_list).unwrap();
    let mut file = File::create(opt.json)?;
    file.write_all(json_content.as_bytes())?;

    let yaml_content = serde_yaml::to_string(&category_list).unwrap();
    let mut file = File::create(opt.yaml)?;
    file.write_all(yaml_content.as_bytes())?;

    Ok(())
}

async fn generate_readme(
    categories: &[Category],
    category_list: &mut HashMap<String, CategoryListItem>,
) -> String {
    let mut toc = String::new();
    let mut tables = String::new();
    for c in categories {
        if category_list.contains_key(&c.id) {
            let mut key = c.title.replace(" ", "-");
            key = key.replace("‌", "");

            toc += &format!("- [{}](#{})\n", c.title, key);

            let channels = category_list.get_mut(&c.id).unwrap();
            channels
                .channels
                .sort_by(|a, b| b.subscriber_count.cmp(&a.subscriber_count));

            tables += &generate_table(&c.title, &channels.channels);
        }
    }

    let mut readme = read_string_file("README.template").await.unwrap();
    readme = readme.replace("{TOC}", &toc);
    readme = readme.replace("{TABLES}", &tables);
    readme
}

fn generate_table(title: &str, channels: &[Channel]) -> String {
    let mut table = format!("## {}\n<table><tbody>", title);

    for ch in channels {
        let p_date = from_gregorian_date(
            ch.updated_at.unwrap().date().year(),
            ch.updated_at.unwrap().date().month() as i32 - 1,
            ch.updated_at.unwrap().date().day() as i32,
        )
        .unwrap();

        table += &format!(
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

    table + "</tbody></table>\n\n"
}

async fn read_string_file(file_name: &str) -> Result<String, String> {
    fs::read_to_string(file_name)
        .await
        .map_err(|e| format!("Read \"{}\" failed with \"{}\" error.", file_name, e))
}

async fn read_json_file<T>(file_name: &str) -> Result<T, String>
where
    T: de::DeserializeOwned,
{
    let content = read_string_file(file_name).await?;
    json5::from_str::<T>(&*content).map_err(|e| {
        format!(
            "Convert json file \"{}\" failed with \"{}\" error.",
            file_name, e
        )
    })
}
