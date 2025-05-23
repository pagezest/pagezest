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

# Measuring RSS/RAM usage of server

There is a shell script that tries to call an API for viewing all the blog posts from the DB. 

The load script makes 50000 calls with a batch of 1000 calls. Server code is written such that it will print the overhead if found any between two consequent calls.

From our observations the average overhead between API calls is 128 kB and overall RSS RAM memory stayed at 8.9 MB for 50000 request ran at a moment.

### Step-1 : Start the Web-Server

```
cargo run --release
```

### Step-2 : Monitor the process memory on system.

In a second terminal window

```
watch -n 1 "ps -o rss,vsz,pid,command -p $(pgrep pagezest)"
```

### Step-3 : Start the Load testing file

In third terminal window

```
chmod +x load.sh
./load.sh
```

### Step-4 : Observe the logs on your Web-Server

In your first terminal window where you had ran pagezest binary in Step-1.


### Step-5 : Start UI dev server
```
cd admin
npm run dev
```

This will start the UI dev server on port 5173 [http://localhost:5173](http://localhost:5173)
