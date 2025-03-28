#!/bin/sh

set -euxo pipefail

version="${1:?Usage: \`\"$0\" <version>\`}"

find . \
    -type f \
    -name 'Cargo.toml' \
    -print \
    -a \
    -exec \
        sed -i -E 's/"(=)?.*"(  # Keep in sync)/"\1'"${version}"'"\2/g' '{}' \
    \; \
;

sed -i -E \
    's@higher-kinded-types/[^/]*/@higher-kinded-types/'"${version}"'/@' \
    README.md \
;

cargo update -v -w
