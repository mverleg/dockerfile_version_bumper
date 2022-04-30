
# Dockerfile version bumper

This is a simple tool for scanning `Dockerfile`s and switching the image tags that appear in `FROM` to their latest version compatible version.

## Command line

`dockerfile_version_bumper` is a small command-line executable that you can run locally or in a CI pipeline.
    
```cli_help
dockerfile_version_bumper 0.0.1
Simple tool for scanning `Dockerfile`s and switching the image tags that appear in `FROM` to their latest version.

USAGE:
    dockerfile_version_bumper [FLAGS] [OPTIONS]

FLAGS:
        --major      Allow bumping to new major versions (which might be incompatible), which is interpreted as the
                     leading number in the version.
        --dry-run    Print the output instead of updating in-place (dry run).
    -h, --help       Prints help information
        --json       Print version bumps in json format. Still bumps Dockerfiles unless --dry-run is also given.
    -V, --version    Prints version information

OPTIONS:
    -f, --dockerfile <dockerfiles>...     [default: Dockerfile]
    -p, --parent <parents>...            Parent images (FROM lines) base names that should be bumped. If empty, bumps
                                         every image in the Dockerfile that is found in the registry.
```
