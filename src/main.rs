use ::std::process::exit;

use ::env_logger;
use ::structopt::StructOpt;
use ::tokio;

use ::dockerfile_version_bumper::bump_dockerfiles;

use crate::args::Args;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

mod args;

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
