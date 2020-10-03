@echo off
cargo build -p conductor-lua --release
lovec conductor-lua-test
