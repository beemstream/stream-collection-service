#!/bin/bash
docker build -t beemstream/stream-collection-service .
docker push beemstream/stream-collection-service
ssh root@157.245.43.172 "docker service update --image beemstream/stream-collection-service beemstream_stream_collection_service"

