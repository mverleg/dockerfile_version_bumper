
# Dockerfile version bumper

This is a simple tool for scanning `Dockerfile`s and switching the image tags that appear in `FROM` to their latest version compatible version.

## Usage

You can [check the releases](https://github.com/mverleg/dockerfile_version_bumper/releases) to see if your platform is included.

If you want to bump Dockerfile versions as part of your CI pipeline, you can use these commands:

```shell
# download the executable and run it
bumper_url="$(curl -s https://api.github.com/repos/mverleg/dockerfile_version_bumper/releases/latest |\
    jq -r '.assets[].browser_download_url | select(. | contains("-x86-64"))' | head -n1)"
curl --silent --location --output dockerfile_version_bumper "$bumper_url"
chmod u+x dockerfile_version_bumper
./dockerfile_version_bumper -f Dockerfile
````

You can change the platform (`x86_64` in the example), pin a specific version (latest in the example), or add any of the flags described below.

## Limitation

Does not support custom repository urls, like `my-repo:8080/user/image`. This is because the url to retrieve tags from isn't known for repos in general, just Dockerhub.

## CLI

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
