use ::std::collections::HashSet;
use ::std::path::PathBuf;

use ::log::debug;

use crate::dvb::data::Parent;
pub use crate::dvb::data::Tag;
use crate::dvb::read::{extract_parents, read_all_dockerfiles};
use crate::dvb::uptag::find_latest_tag;

mod dvb;

/// Unless dry-run, bump all the Dockerfiles for which there is a new matching version.
/// returns: (from-image name, old tag, new tag) if successful, error message otherwise
pub async fn bump_dockerfiles(
    dockerfiles: &[PathBuf],
    allow_parents: &[String],
    bump_major: bool,
    dry_run: bool,
) -> Result<Vec<(String, Tag, Tag)>, String> {
    assert!(bump_major, "bumping only minor versions not implemented, use --major");
    assert!(dry_run, "in-place update not implemented, use --dry-run");
    let dockerfiles = read_all_dockerfiles(dockerfiles).await?;
    let all_parents = extract_parents(&dockerfiles)?;
    let parents = filter_parents(all_parents, allow_parents)?;
    let latest_tags = find_latest_tag(parents, bump_major).await?;
    Ok(latest_tags.into_iter()
        .map(|(parent, new_tag)| (parent.into_name_tag(), new_tag))
        .map(|((name, old_tag), new_tag)| (name, old_tag, new_tag))
        .collect())
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
