#!/bin/bash
set -e
export SERVER_ADDR="http://127.0.0.1:1234"
export AGENT_SOCKET="127.0.0.1:2345"
export RUST_BACKTRACE=1
cargo build
hivemind Procfile
