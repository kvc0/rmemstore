FROM rust:latest as builder

WORKDIR /build
COPY . .

RUN apt-get update
RUN apt-get install -y protobuf-compiler

RUN cargo build --release --bin rmemstored
RUN cargo build --release --bin rms

FROM rust:slim
COPY --from=builder /build/target/release/rmemstored /usr/local/bin/rmemstored
COPY --from=builder /build/target/release/rms /usr/local/bin/rms

CMD [ "rmemstored", "--size", "1gib", "plaintext", "0.0.0.0:9446" ]
