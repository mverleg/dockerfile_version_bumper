use ::std::fs;

use ::indexmap::IndexMap;

use crate::dvb::data::Tag;
use crate::Parent;

pub async fn update_all_dockerfiles(latest_tags: &IndexMap<Parent, Tag>) -> Result<(), String> {
    //TODO @mark: what if multiple updates to same Dockerfile?
    for (parent, new_tag) in latest_tags.iter() {
        let content = format!("# updated: {} {} -> {}!\n{}", parent.name(), parent.tag(), new_tag, parent.dockerfile().content());
        fs::write(parent.dockerfile().path(), content)
            .map_err(|_| format!("failed to update Dockerfile '{}'", parent.name()))?
    }
    unimplemented!()  //TODO @mark: TEMPORARY! REMOVE THIS!
}

