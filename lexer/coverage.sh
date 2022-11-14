#! /usr/bin/bash

RUSTFLAGS="-C instrument-coverage" \
    cargo test --tests

llvm-profdata merge -sparse default_*.profraw -o coverage.profdata
rm default_*.profraw

object=$(
    for file in \
        $(
            RUSTFLAGS="-C instrument-coverage" \
                cargo test --tests --no-run --message-format=json |
                jq -r "select(.profile.test == true) | .filenames[]" |
                grep -v dSYM -
        ); do
        printf "%s %s " -object "$file"
    done
)

llvm-cov show \
    --use-color --ignore-filename-regex='/.cargo/registry' \
    --instr-profile=coverage.profdata \
    $object \
    --show-instantiations --show-line-counts-or-regions \
    --Xdemangler="$HOME/.cargo/bin/rustfilt"
