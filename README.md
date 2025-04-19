Setup Rust on your system.

Run the server

```
cargo run --release
```

This will start the server on port 8080 [http://localhost:8080](http://localhost:8080)

And try hitting the APIs from given [postman collection](https://api.postman.com/collections/21491030-c6505855-123e-43fd-b9da-629adb7c03e2?access_key=PMAT-01JRKD5J1BKS04ENMDBR825G97)


# Load Testing the Server

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
