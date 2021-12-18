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

pub async fn find_latest_tag(parents: HashSet<Parent>, bump_major: bool) -> Result<HashMap<Parent, Vec<String>>, String> {
    let client = Client::new();

    let tags = stream::iter(parents.into_iter()
        .map(|parent| (format!("https://registry.hub.docker.com/v1/repositories/{}/tags", &parent.name()), parent)))
        .map(|(url, parent)| load_filter_tags(parent, &client, url, bump_major))
        .buffer_unordered(16)
        .try_collect::<Vec<_>>()
        .await?;

    unimplemented!()
}

async fn load_filter_tags(parent: Parent, client: &Client, url: String, bump_major: bool) -> Result<(Parent, Tag), String> {
    let data = request_tag_json(client, &url).await?;
    let tag = find_highest(&parent, &data, bump_major)?;
    Ok((parent, tag))
}

fn find_highest(parent: &Parent, data: &String, bump_major: bool) -> Result<Tag, String> {
    let tag = NAME_TAG_RE.captures_iter(&data)
        .filter(|tag| parent.tag_pattern().is_match(&tag[0]))
        .map(|tag| parse_tag(parent.tag_pattern(), &tag[0]).unwrap())
        .filter(|tag| tag >= parent.tag())
        .filter(|tag| bump_major || tag.major() == parent.tag().major())
        .inspect(|tag| debug!("tag = {}", tag))  //TODO @mark: TEMPORARY! REMOVE THIS!
        .sorted()
        .inspect(|tag| debug!("CHOSEN = {}", tag))  //TODO @mark: TEMPORARY! REMOVE THIS!
        .rev()
        .next()
        .ok_or_else(|| format!("could not find the version {} nor any higher ones", parent.tag()))?;
    Ok(tag)
}

async fn request_tag_json(client: &Client, url: &String) -> Result<String, String> {
    debug!("request to: {}", &url);
    let resp = client.get(url).send().await.map_err(|err|
        format!("Failed to request available Docker image tags: err {} for {}", err, &url))?;
    let data = resp.text().await.map_err(|err|
        format!("Failed to request available Docker image tags: err {} for {}", err, &url))?;
    Ok(data)
}
