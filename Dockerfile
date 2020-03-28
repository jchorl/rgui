FROM rust:1.42

RUN apt-get update && apt-get install -y \
    ripgrep
