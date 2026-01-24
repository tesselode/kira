set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]
set dotenv-load

check:
    cargo clippy -p kira --no-default-features
    cargo clippy -p kira --no-default-features --features=mp3 --target=wasm32-unknown-unknown

test:
    cargo test -p kira --all-features
    cargo test -p kira --no-default-features --lib
    cargo test -p kira --no-default-features --features=cpal --lib
    cargo test -p kira --no-default-features --features=mp3 --lib

lint:
    cargo clippy --fix --allow-dirty
    cargo fix --allow-dirty
    cargo fmt