#!/bin/bash

cargo bump patch

version=$(awk -F'[ ="]+' '$1 == "version" { print $2 }' Cargo.toml)

docker build -t beemstream/stream-collection-service:$version .
docker push beemstream/stream-collection-service:$version

# Tag image as latest
docker pull beemstream/stream-collection-service:$version
docker tag beemstream/stream-collection-service:$version beemstream/stream-collection-service:latest
docker push beemstream/stream-collection-service:latest

ssh root@157.245.43.172 "docker service update --image beemstream/stream-collection-service beemstream_stream_collection_service && docker container prune -f && docker image prune -a -f"
