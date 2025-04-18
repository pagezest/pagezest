# Pagezest docker container

## Manual setup
1. Build the server binary `cargo build [--release]`, and copy the binary to this directory.
2. Copy plugins files(wasm and json) to this directory.
3. Build the image with `docker build -t pagezest .`
4. Run the container with `docker run -p 8080:8080 pagezest:latest`
