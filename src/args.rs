use ::std::path::PathBuf;

use ::structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "dockerfile_version_bumper",
    about = "Simple tool for scanning `Dockerfile`s and switching the image tags that appear in `FROM` to their latest version."
)]
pub struct Args {
    #[structopt(long = "dockerfile", short = "f", default_value = "Dockerfile", parse(from_os_str))]
    pub evolution_dir: Vec<PathBuf>,
    #[structopt(long = "parent", short = "p", help = "Parent images (FROM lines) base names that should be bumped. If empty, bumps every image in the Dockerfile that is found in the registry.")]
    pub parent: Vec<String>,
    #[structopt(long, help = "Allow bumping to new major versions (which might be incompatible), which is interpreted as the leading number in the version.")]
    pub major: bool,
}
