use ::std::collections::HashMap;
use ::std::collections::HashSet;

use ::futures::{stream, StreamExt, TryStreamExt};
use ::itertools::Itertools;
use ::lazy_static::lazy_static;
use ::log::debug;
use ::regex::Regex;
use ::reqwest::Client;

use crate::dvb::data::{parse_tag, Tag};
use crate::Parent;

lazy_static! {
    static ref NAME_TAG_RE: Regex = Regex::new("\"name\":\\s*\"([^\"]*)\"").unwrap();
}

pub async fn find_available_tags(parents: HashSet<Parent>) -> Result<HashMap<Parent, Vec<String>>, String> {
    let client = Client::new();

    let tags = stream::iter(parents.into_iter()
        .map(|parent| (format!("https://registry.hub.docker.com/v1/repositories/{}/tags", &parent.name()), parent)))
        .map(|(url, parent)| find_highest_tag(parent, &client, url))
        .buffer_unordered(16)
        .try_collect::<Vec<_>>()
        .await?;

    dbg!(tags);
    // .map();
    //.into_future().await;
    // https://stackoverflow.com/questions/46041185/how-do-i-append-futures-to-a-bufferunordered-stream
    //.into_iter()
    //.and_then(|(tags, parent)| (parent, choose_tag(&tags, &parent)));
    //.collect::<Result<Vec<_>, String>>()
    // .collect();
    // .await;

    // .collect::<Result<Vec<_>, String>>()
    // .await;

    // .map(|(parent, tags)| (parent, choose_tag(&parent, &tags)))
    // .collect::<Result<Vec<_>>>();

    unimplemented!()
}

async fn find_highest_tag(parent: Parent, client: &Client, url: String) -> Result<(Parent, Tag), String> {
    let data = request_tag_json(client, &url).await?;
    let tag = NAME_TAG_RE.captures_iter(&data)
        .filter(|tag| parent.tag_pattern().is_match(&tag[0]))
        .map(|tag| parse_tag(parent.tag_pattern(), &tag[0]).unwrap())
        .filter(|tag| tag >= parent.tag())
        .sorted()
        .rev()
        .next()
        .ok_or_else(|| format!("could not find the version {} nor any higher ones", parent.tag()))?;
    Ok((parent, tag))
}

async fn request_tag_json(client: &Client, url: &String) -> Result<String, String> {
    debug!("request to: {}", &url);
    let resp = client.get(url).send().await.map_err(|err|
        format!("Failed to request available Docker image tags: err {} for {}", err, &url))?;
    let data = resp.text().await.map_err(|err|
        format!("Failed to request available Docker image tags: err {} for {}", err, &url))?;
    Ok(data)
}
