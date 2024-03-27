#!/bin/sh
exec cargo run \
    --quiet \
    --release \
    --target-dir=/tmp/dwt \
    --manifest-path $(dirname $0)/Cargo.toml \
    -- "$@"
