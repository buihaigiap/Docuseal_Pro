#!/bin/bash

# Script to start MinIO and run DocuSeal Pro

set -e

echo "Checking if MinIO container is running..."
if ! docker ps | grep -q minio; then
    echo "Starting MinIO container..."
    docker run -d -p 9000:9000 -p 9001:9001 --name minio \
        -e "MINIO_ACCESS_KEY=minioadmin" \
        -e "MINIO_SECRET_KEY=minioadmin" \
        minio/minio server /data --console-address ":9001" > /dev/null 2>&1

    echo "Waiting for MinIO to start..."
    sleep 5

    echo "Creating docuseal bucket..."
    docker exec minio mc alias set myminio http://localhost:9000 minioadmin minioadmin > /dev/null 2>&1
    docker exec minio mc mb myminio/docuseal > /dev/null 2>&1
    
    echo "Setting public access policy for docuseal bucket..."
    docker exec minio mc anonymous set public myminio/docuseal > /dev/null 2>&1
    
    echo "MinIO started and bucket created with public access."
else
    echo "MinIO is already running."
fi

echo "Starting DocuSeal Pro..."
echo "Loading environment variables from .env..."
set -a
source .env
set +a
cargo run