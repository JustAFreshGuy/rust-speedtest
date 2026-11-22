# Rust Speedtest

A small asynchronous Rust command-line program that measures HTTP request latency, download throughput, and upload throughput using `reqwest` and Tokio.

## Run

```sh
cargo run --release
```

The current implementation uses the public LibreSpeed test server in Nuremberg, Germany:

- Ping and upload: `https://de3.backend.librespeed.org/empty.php`
- Download: `https://de3.backend.librespeed.org/garbage.php`

The server is part of LibreSpeed's public server list. For a website used by others, host your own speed-test server so test traffic and capacity are under your control.

