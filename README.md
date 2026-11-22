# Rust Speedtest

A small asynchronous Rust command-line program that measures HTTP request latency, download throughput, and upload throughput using `reqwest` and Tokio.

## Run

```sh
cargo run --release
```

The current implementation sends its measurements to `https://cloudflare.com`.

