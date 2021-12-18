use ::std::collections::HashMap;
use ::std::collections::HashSet;
use ::std::path::PathBuf;

use ::futures::{FutureExt, stream, StreamExt, TryStreamExt};
use ::log::debug;
use ::reqwest::Client;

use crate::dvb::data::Parent;
use crate::dvb::read::{extract_parents, read_all_dockerfiles};
use crate::dvb::uptag::find_available_tags;

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
