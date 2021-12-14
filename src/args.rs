use ::structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "dockerfile_version_bumper",
    about = "Simple tool for scanning `Dockerfile`s and switching the image tags that appear in `FROM` to their latest version."
)]
pub struct Args {
    #[structopt(long = "dockerfile", short = "f")]
    pub evolution_dir: String,
}
