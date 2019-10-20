FROM rustlang/rust:nightly as build

RUN USER=root cargo new rs-bot
COPY Cargo.toml Cargo.lock /usr/src/rs-bot/

WORKDIR /usr/src/rs-bot
RUN mkdir -p src/bin && echo "fn main() { }" > src/bin/main.rs
RUN echo "" > src/lib.rs

RUN cargo build --release

COPY src /usr/src/rs-bot/src
RUN rm target/release/trade-history
RUN cargo build --release \
    && mv target/release/trade-history /bin/ \
    && rm -rf /usr/src/rs-bot

WORKDIR /

CMD ["/bin/trade-history"]
