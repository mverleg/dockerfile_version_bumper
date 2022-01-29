use ::std::collections::HashMap;
use ::std::fs;
use ::std::path::PathBuf;
use std::borrow::Borrow;

use ::indexmap::IndexMap;

use crate::dvb::data::Tag;
use crate::Parent;

pub async fn update_all_dockerfiles(latest_tags: &IndexMap<Parent, Tag>) -> Result<(), String> {
    let new_content = updated_dockerfiles_content(latest_tags);
    unimplemented!()  //TODO @mark: TEMPORARY! REMOVE THIS!
}

fn updated_dockerfiles_content(latest_tags: &IndexMap<Parent, Tag>) -> HashMap<PathBuf, String> {
    //TODO @mark: what if multiple updates to same Dockerfile?
    let mut files = HashMap::new();
    for (parent, new_tag) in latest_tags.iter() {
        let content: &mut &String = files.entry(parent.dockerfile().path())
            .or_insert_with(|| parent.dockerfile().content());
        let q: String = parent.tag_pattern().replace_all(content, format!("{}", new_tag)).into_owned();
        *content = q;
        // let content = format!("# updated: {} {} -> {}!\n{}", parent.name(), parent.tag(), new_tag, parent.dockerfile().content());
        // fs::write(parent.dockerfile().path(), content)
        //     .map_err(|_| format!("failed to update Dockerfile '{}'", parent.name()))?
    }
    unimplemented!()  //TODO @mark: TEMPORARY! REMOVE THIS!
}
