FROM rust:slim-bullseye

WORKDIR /app

COPY tests/envs/rust/ /app/
COPY tests/data/expected_rust.rs /app/src/generated.rs

CMD cargo test
