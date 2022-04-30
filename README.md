
# Dockerfile version bumper

This is a simple tool for scanning `Dockerfile`s and switching the image tags that appear in `FROM` to their latest version compatible version.

## Command line

`dockerfile_version_bumper` is a small command-line executable that you can run locally or in a CI pipeline.
    
```cli_help
    FLAGS:
            --major      Allow bumping to new major versions (which might be incompatible), 
                         which is interpreted as the leading number in the version.
            --dry-run    Print the output instead of updating in-place (dry run).
            --json       Print version bumps in json format. Still bumps Dockerfiles unless 
                         --dry-run is also given.
    
    OPTIONS:
        -f, --dockerfile <dockerfiles>...  [default: Dockerfile]
        -p, --parent <parents>...          Only bump named parents [default: every FROM lime]
```
