use ::std::collections::HashSet;

use ::futures::{stream, StreamExt, TryStreamExt};
use ::indexmap::IndexMap;
use ::itertools::Itertools;
use ::lazy_static::lazy_static;
use ::log::debug;
use ::regex::Regex;
use ::reqwest::Client;
use crate::dvb::convert::parse_tag;
use crate::dvb::data::Tag;

use crate::Parent;

lazy_static! {
    static ref NAME_TAG_RE: Regex = Regex::new("\"name\":\\s*\"([^\"]*)\"").unwrap();
}

pub async fn find_latest_tag(
    parents: HashSet<Parent>,
    bump_major: bool,
) -> Result<IndexMap<Parent, Tag>, String> {
    let client = Client::new();

    let latest_tags = stream::iter(parents.into_iter().map(|parent| {
        (
            format!(
                "https://registry.hub.docker.com/v1/repositories/{}/tags",
                &parent.image_name()
            ),
            parent,
        )
    }))
    .map(|(url, parent)| load_filter_tags(parent, &client, url, bump_major))
    .buffer_unordered(8)
    .try_collect::<Vec<_>>()
    .await?;

    Ok(latest_tags
        .into_iter()
        .sorted_by(|(parent1, _), (parent2, _)| {
            parent1
                .dockerfile()
                .cmp(parent2.dockerfile())
                .then(parent1.image_name().cmp(parent2.image_name()))
        })
        .collect::<IndexMap<Parent, Tag>>())
}

async fn load_filter_tags(
    parent: Parent,
    client: &Client,
    url: String,
    bump_major: bool,
) -> Result<(Parent, Tag), String> {
    let data = request_tag_json(client, &url).await?;
    let tag = find_highest(&parent, &data, bump_major)?;
    Ok((parent, tag))
}

fn find_highest(parent: &Parent, data: &str, bump_major: bool) -> Result<Tag, String> {
    let tag = NAME_TAG_RE
        .captures_iter(data)
        .filter(|tag| parent.tag_pattern().is_match(&tag[1]))
        .map(|tag| parse_tag(parent.tag_pattern(), &tag[1]).unwrap())
        .filter(|tag| tag >= parent.tag())
        .filter(|tag| bump_major || tag.major() == parent.tag().major())
        //.inspect(|tag| debug!("tag = {}", tag))  //TODO @mark: TEMPORARY! REMOVE THIS!
        .sorted()
        //.inspect(|tag| debug!("CHOSEN = {}", tag))  //TODO @mark: TEMPORARY! REMOVE THIS!
        .rev()
        .next()
        .ok_or_else(|| {
            format!(
                "could not find the version {} nor any higher ones",
                parent.tag()
            )
        })?;
    Ok(tag)
}

async fn request_tag_json(client: &Client, url: &str) -> Result<String, String> {
    debug!("request to: {}", &url);
    let resp = client.get(url).send().await.map_err(|err| {
        format!(
            "Failed to request available Docker image tags: err {} for {}",
            err, &url
        )
    })?;
    let data = resp.text().await.map_err(|err| {
        format!(
            "Failed to request available Docker image tags: err {} for {}",
            err, &url
        )
    })?;
    Ok(data)
}

#[cfg(test)]
mod tests {
    use ::std::path::PathBuf;
    use ::std::rc::Rc;

    use crate::dvb::data::Dockerfile;

    use super::*;

    static TAGS_JSON: &str = "[\
            {\"layer\": \"\", \"name\": \"2.5.1-full\"}, \
            {\"layer\": \"\", \"name\": \"3.6.6-full\"}, \
            {\"layer\": \"\", \"name\": \"3.6.6-alpine-perl\"}, \
            {\"layer\": \"\", \"name\": \"2.4.1-alpine\"}, \
            {\"layer\": \"\", \"name\": \"3.5.2-alpine\"}, \
            {\"layer\": \"\", \"name\": \"1.9.9-alpine\"}]";

    #[test]
    fn bump_minor() {
        let dockerfile = Rc::new(Dockerfile::new(PathBuf::from("file.ext"), "".to_owned()));
        let parent = Parent::new(
            dockerfile,
            "".to_owned(),
            Regex::new(r"^([0-9]+)\.([0-9]+)\.([0-9]+)\-alpine$").unwrap(),
            Tag::new("2.2.8-alpine".to_owned(), (2, 2, 8, 0)),
            "AS build".to_owned(),
        );
        let highest = find_highest(&parent, TAGS_JSON, false);
        assert_eq!(
            highest,
            Ok(Tag::new("2.4.1-alpine".to_owned(), (2, 4, 1, 0)))
        );
    }

    #[test]
    fn bump_major() {
        let dockerfile = Rc::new(Dockerfile::new(PathBuf::from("file.ext"), "".to_owned()));
        let parent = Parent::new(
            dockerfile,
            "".to_owned(),
            Regex::new(r"^([0-9]+)\.([0-9]+)\.([0-9]+)\-alpine$").unwrap(),
            Tag::new("2.2.8-alpine".to_owned(), (2, 2, 8, 0)),
            "AS build".to_owned(),
        );
        let highest = find_highest(&parent, TAGS_JSON, true);
        assert_eq!(
            highest,
            Ok(Tag::new("3.5.2-alpine".to_owned(), (3, 5, 2, 0)))
        );
    }
}
