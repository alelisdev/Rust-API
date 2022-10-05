FROM rust:1.51 as builder
WORKDIR /usr/src/primecrime-api
COPY . .
RUN cargo install --path . --locked

FROM debian:buster-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates openssl
COPY --from=builder /usr/local/cargo/bin/primecrime-api /usr/local/bin/primecrime-api
EXPOSE 3030
CMD ["primecrime-api"]
