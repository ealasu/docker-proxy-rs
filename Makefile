all: build docker

build:
	docker run --rm -it -v $(shell pwd):/data -w /data -v ~/.cargo/registry:/root/.cargo/registry liuchong/rustup:stable cargo build

docker:
	docker build -t docker-proxy-rs:latest .
