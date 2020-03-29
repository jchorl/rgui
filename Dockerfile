FROM rust:1.42

RUN apt-get update && apt-get install -y \
    librust-clang-sys-dev \
    ripgrep

RUN rustup component add rustfmt
