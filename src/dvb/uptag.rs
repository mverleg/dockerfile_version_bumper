use ::std::collections::HashMap;
use ::std::collections::HashSet;
use ::std::path::PathBuf;
use ::std::str::from_utf8;
use ::bytes::Bytes;

use ::futures::{FutureExt, stream, StreamExt, TryStreamExt};
use ::log::debug;
use ::regex::bytes;
use ::reqwest::Client;

use crate::Parent;

pub async fn find_available_tags(parents: HashSet<Parent>) -> Result<HashMap<Parent, Vec<String>>, String> {
    let client = Client::new();

    let tags = stream::iter(parents.into_iter()
        .map(|parent| (format!("https://registry.hub.docker.com/v1/repositories/{}/tags", &parent.name()), parent)))
        .map(|(url, parent)| load_tags(&client, url))
        //.map(|(tags, parent)| choose_tag(parent, tags))
        .take(1)  //TODO @mark: TEMPORARY! REMOVE THIS!
        .buffer_unordered(16)
        .try_collect::<Vec<_>>()
        .await;

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

async fn load_tags(client: &Client, url: String) -> Result<Vec<String>, String> {
    let data = request_tag_json(client, &url).await?;
    //TODO @mark: parse json

    dbg!(data);
    Ok(vec![])  //TODO @mark: TEMPORARY! REMOVE THIS!
}

async fn request_tag_json(client: &Client, url: &String) -> Result<bytes::Bytes, String> {
    debug!("request to: {}", &url);
    let resp = client.get(url).send().await.map_err(|err|
        format!("Failed to request available Docker image tags: err {} for {}", err, &url))?;
    let data = resp.bytes().await.map_err(|err|
        format!("Failed to request available Docker image tags: err {} for {}", err, &url))?;
    Ok(data)
}

fn choose_tag(parent: &Parent, tags: &str) {
    unimplemented!()
}
