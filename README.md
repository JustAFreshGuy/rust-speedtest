# Rust Speedtest

A small asynchronous Rust command-line program that measures HTTP request latency, download throughput, and upload throughput using `reqwest` and Tokio.

## Run

```sh
cargo run --release
```

The current implementation uses the public LibreSpeed test server in Prague, Czech Republic:

- Ping and upload: `https://librespeed.turris.cz/backend/empty.php`
- Download: `https://librespeed.turris.cz/backend/garbage.php`

The server is part of LibreSpeed's public server list. For a website used by others, host your own speed-test server so test traffic and capacity are under your control.

