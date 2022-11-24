#!/bin/sh

cargo install cargo-deny cargo-outdated cargo-audit cargo-pants

cargo fmt --all -- --check

cargo clippy -- -D warnings

cargo deny check advisories bans sources

cargo outdated

rm -rf ~/.cargo/advisory-db

cargo audit

cargo pants
