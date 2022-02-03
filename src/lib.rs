use ::std::collections::HashSet;
use ::std::path::PathBuf;

use ::derive_new::new;
use ::log::debug;

use crate::dvb::data::Parent;
use crate::dvb::read::{extract_parents, read_all_dockerfiles};
use crate::dvb::uptag::find_latest_tag;
use crate::dvb::write::update_all_dockerfiles;

mod dvb;

/// Unless dry-run, bump all the Dockerfiles for which there is a new matching version.
/// returns: (dockerfile path, from-image name, old tag, new tag) if successful, error message otherwise
pub async fn bump_dockerfiles(
    dockerfiles: &[PathBuf],
    allow_parents: &[String],
    bump_major: bool,
    dry_run: bool,
) -> Result<Vec<TagUp>, String> {
    assert!(
        bump_major,
        "bumping only minor versions not implemented, use --major"
    );
    assert!(dry_run, "in-place update not implemented, use --dry-run");
    let dockerfiles = read_all_dockerfiles(dockerfiles).await?;
    let all_parents = extract_parents(&dockerfiles)?;
    let parents = filter_parents(all_parents, allow_parents)?;
    let latest_tags = find_latest_tag(parents, bump_major).await?;
    update_all_dockerfiles(&latest_tags).await?;
    Ok(latest_tags
        .into_iter()
        .map(|(parent, new_tag)| (parent.explode(), new_tag))
        .map(|((dockerfile, name, old_tag), new_tag)| {
            TagUp::new(
                dockerfile,
                name,
                old_tag.name().to_owned(),
                new_tag.name().to_owned(),
            )
        })
        .collect())
}

#[derive(Debug, Clone, new)]
pub struct TagUp {
    pub dockerfile: PathBuf,
    pub image: String,
    pub old_tag: String,
    pub new_tag: String,
}

fn filter_parents(
    all_parents: HashSet<Parent>,
    allow_parent_names: &[String],
) -> Result<HashSet<Parent>, String> {
    if allow_parent_names.is_empty() {
        if all_parents.is_empty() {
            return Err("No FROM tags with versions were found in the Dockerfile(s)".to_owned());
        }
        return Ok(all_parents);
    }
    let allow_parent_names: HashSet<String> =
        HashSet::from_iter(allow_parent_names.iter().cloned());
    let parents = all_parents
        .into_iter()
        .filter(|parent| allow_parent_names.contains(parent.image_name()))
        .inspect(|parent| debug!("including parent (-p): {}", parent))
        .collect::<HashSet<_>>();
    if parents.is_empty() {
        return Err(
            "None of the FROM tags given with --parent/-p were found in the Dockerfile(s)"
                .to_owned(),
        );
    }
    Ok(parents)
}
