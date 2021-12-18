use ::std::collections::HashMap;
use ::std::collections::HashSet;
use ::std::path::PathBuf;

use ::futures::{FutureExt, stream, StreamExt, TryFutureExt, TryStreamExt};
use ::log::debug;
use ::reqwest::Client;

use crate::dvb::data::Parent;
use crate::dvb::read::{extract_parents, read_all_dockerfiles};

mod dvb;

pub async fn bump_dockerfiles(
    dockerfiles: &[PathBuf],
    allow_parents: &[String],
    bump_major: bool,
    print: bool,
) -> Result<(), String> {
    assert!(bump_major, "bumping only minor versions not implemented, use --major");
    assert!(print, "in-place update not implemented, use --print");
    let dockerfiles = read_all_dockerfiles(dockerfiles).await?;
    let all_parents = extract_parents(&dockerfiles)?;
    let parents = filter_parents(all_parents, allow_parents)?;
    let available_tags = find_available_tags(parents).await?;
    unimplemented!()
}

fn filter_parents(all_parents: HashSet<Parent>, allow_parent_names: &[String]) -> Result<HashSet<Parent>, String> {
    if allow_parent_names.is_empty() {
        if all_parents.is_empty() {
            return Err("No FROM tags with versions were found in the Dockerfile(s)".to_owned())
        }
        return Ok(all_parents)
    }
    let allow_parent_names: HashSet<String> = HashSet::from_iter(allow_parent_names.iter().cloned());
    let parents = all_parents.into_iter()
        .filter(|parent| allow_parent_names.contains(parent.name()))
        .inspect(|parent| debug!("including parent (-p): {}", parent))
        .collect::<HashSet<_>>();
    if parents.is_empty() {
        return Err("None of the FROM tags given with --parent/-p were found in the Dockerfile(s)".to_owned())
    }
    Ok(parents)
}

async fn find_available_tags(parents: HashSet<Parent>) -> Result<HashMap<Parent, Vec<String>>, String> {
    let client = Client::new();

    let future = async { 1 };
    let new_future = future.map(|x| x + 3);

    // let q = load_tags(&client, "https://registry.hub.docker.com/v1/repositories/".to_owned())
    //     .map(|future_content| ()).await;

    let tags = stream::iter(parents.into_iter()
        .map(|parent| (format!("https://registry.hub.docker.com/v1/repositories/{}/tags", &parent.name()), parent)))
        // .map(|(url, parent)| load_tags(&client, url))
        //.then(|(url, parent)| load_tags(&client, url).map(|tags_res| tags_res.map(|tags| (parent, tags))))
        //.then(|(url, parent)| load_tags(&client, url).map(|tags_res| (parent, tags_res.unwrap())))
        .map(|(url, parent)| load_tags(&client, url)
            .map(|tags_res| tags_res.map(|tags| (tags, parent))))
        //TODO @mark: increase to 16
        .buffer_unordered(2)
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
    let resp = client.get(&url).send().await.map_err(|err|
        format!("Failed to request available Docker image tags: err {} for {}", err, &url))?;
    let data = resp.bytes().await.map_err(|err|
        format!("Failed to request available Docker image tags: err {} for {}", err, &url))?;
    //TODO @mark: parse json
    Ok(vec![])  //TODO @mark: TEMPORARY! REMOVE THIS!
}

fn choose_tag(parent: &Parent, tags: &str) {
    unimplemented!()
}
