use bytes::Bytes;
use futures_util::StreamExt;
use reqwest::Client;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

const PING_URL: &str = "https://librespeed.turris.cz/backend/empty.php";
const DOWN_URL: &str = "https://librespeed.turris.cz/backend/garbage.php";
const UP_URL: &str = "https://librespeed.turris.cz/backend/empty.php";
const CONCURRENCY: usize = 4;
const TEST_DURATION: Duration = Duration::from_secs(8);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .build()?;

    println!("=== Rust Speedtest ===\n");

    // 1. Ping & Jitter Section
    println!("[1/3] Measuring Ping & Jitter...");
    let (avg_ping, min_ping, max_ping, jitter) = measure_ping(&client, 10).await;
    println!(
        "  Avg Ping : {:.2} ms (Min: {:.2} ms | Max: {:.2} ms)",
        avg_ping, min_ping, max_ping
    );
    println!("  Jitter   : {:.2} ms\n", jitter);

    // 2. Download Speed Section
    println!("[2/3] Testing Download Speed ({CONCURRENCY} streams)...");
    let down_mbps = measure_download(&client).await;
    println!("  Download : {:.2} Mbps\n", down_mbps);

    // 3. Upload Speed Section
    println!("[3/3] Testing Upload Speed ({CONCURRENCY} streams)...");
    let up_mbps = measure_upload(&client).await;
    println!("  Upload   : {:.2} Mbps\n", up_mbps);

    println!("=== Summary ===");
    println!("Ping: {:.2} ms | Jitter: {:.2} ms | Down: {:.2} Mbps | Up: {:.2} Mbps",
        avg_ping, jitter, down_mbps, up_mbps
    );

    Ok(())
}

async fn measure_ping(client: &Client, samples: usize) -> (f64, f64, f64, f64) {
    // Warmup connection (TLS handshake)
    let _ = client.get(PING_URL).send().await;

    let mut rtts = Vec::with_capacity(samples);
    for _ in 0..samples {
        let start = Instant::now();
        if let Ok(resp) = client.get(PING_URL).send().await {
            let _ = resp.bytes().await;
            rtts.push(start.elapsed().as_secs_f64() * 1000.0);
        }
        tokio::time::sleep(Duration::from_millis(150)).await;
    }

    if rtts.is_empty() {
        return (0.0, 0.0, 0.0, 0.0);
    }

    let sum: f64 = rtts.iter().sum();
    let avg = sum / rtts.len() as f64;
    let min = rtts.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = rtts.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    // Average absolute difference between consecutive pings
    let jitter = if rtts.len() > 1 {
        let diffs: f64 = rtts.windows(2).map(|w| (w[1] - w[0]).abs()).sum();
        diffs / (rtts.len() - 1) as f64
    } else {
        0.0
    };

    (avg, min, max, jitter)
}

async fn measure_download(client: &Client) -> f64 {
    let total_bytes = Arc::new(AtomicU64::new(0));
    let start = Instant::now();
    let mut handles = Vec::new();

    for _ in 0..CONCURRENCY {
        let client = client.clone();
        let counter = Arc::clone(&total_bytes);

        handles.push(tokio::spawn(async move {
            while start.elapsed() < TEST_DURATION {
                if let Ok(resp) = client.get(DOWN_URL).send().await {
                    let mut stream = resp.bytes_stream();
                    while let Some(Ok(chunk)) = stream.next().await {
                        counter.fetch_add(chunk.len() as u64, Ordering::Relaxed);
                        if start.elapsed() >= TEST_DURATION {
                            break;
                        }
                    }
                }
            }
        }));
    }

    for h in handles {
        let _ = h.await;
    }

    let elapsed = start.elapsed().as_secs_f64();
    let bytes = total_bytes.load(Ordering::Relaxed) as f64;
    (bytes * 8.0) / elapsed / 1_000_000.0
}

async fn measure_upload(client: &Client) -> f64 {
    let total_bytes = Arc::new(AtomicU64::new(0));
    // 1 MB payload chunk reused across requests
    let payload = Bytes::from(vec![0u8; 1_000_000]);
    let start = Instant::now();
    let mut handles = Vec::new();

    for _ in 0..CONCURRENCY {
        let client = client.clone();
        let counter = Arc::clone(&total_bytes);
        let data = payload.clone();

        handles.push(tokio::spawn(async move {
            while start.elapsed() < TEST_DURATION {
                let len = data.len() as u64;
                if client.post(UP_URL).body(data.clone()).send().await.is_ok() {
                    counter.fetch_add(len, Ordering::Relaxed);
                }
            }
        }));
    }

    for h in handles {
        let _ = h.await;
    }

    let elapsed = start.elapsed().as_secs_f64();
    let bytes = total_bytes.load(Ordering::Relaxed) as f64;
    (bytes * 8.0) / elapsed / 1_000_000.0
}

