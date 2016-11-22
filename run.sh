#!/usr/bin/env bash

#exec docker run --rm -it -v $HOME/.docker/cache:/var/docker-cache -v $HOME/.docker/cache-config.json:/etc/docker-cache/config.json docker-proxy-rs:latest
exec docker run --rm -it  -v $HOME/.docker/cache-config.json:/etc/docker-cache/config.json docker-proxy-rs:latest
