#!/bin/bash
set -e
cargo clippy -- -W clippy::redundant-else -W clippy::needless-pass-by-value -W clippy::missing-panics-doc -W clippy::single-match-else
cargo deny check