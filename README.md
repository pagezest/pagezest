# How to Compile the Server

Installation Requirements:
* Rust toolchain (rustc, cargo): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
* libsqlite: `apt install libsqlite3-dev`

Do steps 1-3. You need all steps working in order for the server to work.

1. Compile the UI: `sh scripts/test.sh ui`
2. Compile the Plugins: `sh scripts/test.sh p`
3. Compile the Server: `sh scripts/test.sh b`

# Measuring requests per second of server

Just run Apache Benchmark: `ab -n 15000 -c 1000 "http://$IP:8080/hello-world"`
