use ::std::path::PathBuf;
use ::std::process::exit;

use ::env_logger;
use ::structopt::StructOpt;
use ::tokio;

use ::dockerfile_version_bumper::bump_dockerfiles;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[derive(Debug, StructOpt)]
#[structopt(
name = "dockerfile_version_bumper",
about = "Simple tool for scanning `Dockerfile`s and switching the image tags that appear in `FROM` to their latest version."
)]
pub struct Args {
    #[structopt(long = "dockerfile", short = "f", default_value = "Dockerfile", parse(from_os_str))]
    pub dockerfiles: Vec<PathBuf>,
    #[structopt(long = "parent", short = "p", help = "Parent images (FROM lines) base names that should be bumped. If empty, bumps every image in the Dockerfile that is found in the registry.")]
    pub parents: Vec<String>,
    #[structopt(long = "major", help = "Allow bumping to new major versions (which might be incompatible), which is interpreted as the leading number in the version.")]
    pub bump_major: bool,
    #[structopt(long = "print", short = "d", help = "Print the output instead of updating in-place (dry run).")]
    pub print: bool,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::from_args();
    match bump_dockerfiles(&args.dockerfiles, &args.parents, args.bump_major, args.print).await {
        Ok(()) => {
            eprintln!("done")
        }
        Err(err) => {
            eprintln!("Fatal! {}", err);
            exit(1);
        }
    }
}
