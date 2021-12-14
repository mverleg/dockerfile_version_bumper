use ::std::path::Path;
use ::std::path::PathBuf;
use std::fs::read_to_string;

pub fn bump_dockerfiles(
    dockerfiles: &[PathBuf],
    parents: &[String],
    bump_major: bool,
) -> Result<(), String> {
    dockerfiles.iter()
        .map(|path| read_dockerfile(path.as_path()))
        .collect::<Result<Vec<_>, _>>()?;
    unimplemented!()
}

fn read_dockerfile(path: &Path) -> Result<String, String> {
    read_to_string(path)
        .map_err(|err| format!("Could not read Dockerfile '{}'.\nProvide a correct path using -f PATH.\nError: {}", path.to_string_lossy(), err))
}
