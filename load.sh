#!/bin/bash

echo "Starting Load Testing"

TOTAL_REQ=50000
BATCH_SIZE=1000
IMAGE_LOAD_URL="http://localhost:8080/api/test-image"
URL="http://localhost:8080/api/blogs"

echo "1. Testing loading of an Image from an URL"

for ((i=0; i<100; i++)); do
        echo "$((i)) Image Load"
        curl -s -o /dev/null "$IMAGE_LOAD_URL" &
done

echo "2. Reading one blog from DB"

for ((batch_start=1; batch_start<=TOTAL_REQ; batch_start+=BATCH_SIZE)); do
    for ((i=0; i<BATCH_SIZE; i++)); do
        curl -s -o /dev/null "$URL" &
    done

    wait
    echo "Sent $((batch_start + BATCH_SIZE - 1)) requests"
done

echo "3. Writing 100 blogs into DB"

for ((i=0; i<100; i++)); do
    DATA=$(jq -n --arg title "Sample Blog" --arg content "This is a sample blog content" --arg slug "blog-$i" \
        '{title: $title, content: $content, slug: $slug}')
    curl -s -X POST -H "Content-Type: application/json" -d "$DATA" "http://localhost:8080/api/blog/new" &
    done


echo "4. Reading 100 blogs from DB."

for ((batch_start=1; batch_start<=TOTAL_REQ; batch_start+=BATCH_SIZE)); do
    for ((i=0; i<BATCH_SIZE; i++)); do
        curl -s -o /dev/null "$URL" &
    done

    wait
    echo "Sent $((batch_start + BATCH_SIZE - 1)) requests"
done

echo "Completed Load Testing"
