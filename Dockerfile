FROM --platform=linux/amd64 rust:1.78.0-bookworm as builder

WORKDIR /usr/src/ord

COPY . .

RUN cargo build --bin ord --release

FROM --platform=linux/amd64 debian:bookworm

COPY --from=builder /usr/src/ord/target/release/ord /usr/local/bin/rune

RUN apt-get update && apt-get install -y htop vim less && \
    apt-get install -y curl gnupg ssh rsync iputils-ping procps nfs-common dnsutils telnet net-tools && \
    apt-get install -y jq && \
    apt-get install -y awscli && \
    apt-get clean

ENV RUST_BACKTRACE=1
ENV RUST_LOG=info

ENTRYPOINT ["/usr/local/bin/rune"]
