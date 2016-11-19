FROM debian:stretch-slim

ADD ./config /etc/docker-proxy
ADD ./target/debug/docker-proxy /app/

ENV RUST_BACKTRACE=1
ENV RUST_LOG=info
WORKDIR /app
CMD /app/docker-proxy
