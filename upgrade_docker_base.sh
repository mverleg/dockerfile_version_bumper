#!/usr/bin/env sh

# Finds the FROM statements in Dockerfiles, looks up the latest version on Dockerhub,
# then bumps the versions used in those Dockerfiles.

set -eu

find . -name '*Dockerfile' |
    while IFS= read -r dfile; do

        grep -E '^FROM +[^: ]+:[^ ]*' "$dfile" | sed -E 's/^FROM +([^: ]+):.*/\1/' |
            while IFS= read -r img; do

                img_esc="$(printf '%s' "$img" | sed 's/\//\\\//')"
                oldtag="$(grep -E "^FROM +${img_esc}:[^ ]*" "$dfile" | sed -E "s/^FROM +${img_esc}:([^ ]*).?*/\1/")"
                if [ "$oldtag" = "" ] || [ "$oldtag" = "null" ]; then
                    printf 'could not find current docker image oldtag for "%s"\n' "$img"
                    exit 1;
                fi
                tag_re="$(printf '%s' "$oldtag" | sed 's/\./\\\./' | sed 's/\-/\\\-/' | sed 's/\//\\\//' | sed -E 's/[0-9]+/.*/g')"

                newtag="$(curl -sS -L "https://registry.hub.docker.com/v1/repositories/${img}/tags" |
                    jq -r ".[] | .name" |
                    grep -E "^${tag_re}$" |
                    sort --version-sort |
                    tail -n1)"
                if [ "$newtag" = "" ] || [ "$newtag" = "null" ]; then
                    printf 'could not find new docker image oldtag for "%s" (oldtag regex: "%s")\n' "$img" "$tag_re"
                    exit 1;
                fi

                if [ "$newtag" = "$oldtag" ]; then
                    printf '%s: "%s" "%s" unchanged\n' "$dfile" "$img" "$oldtag"
                else
                    printf '%s: "%s" "%s" -> "%s"\n' "$dfile" "$img" "$oldtag" "$newtag"
                fi

                sed -i'' -E "s/${img_esc}:[^ ]*/${img_esc}:$newtag/" "$dfile"
            done
    done

printf 'done\n'


