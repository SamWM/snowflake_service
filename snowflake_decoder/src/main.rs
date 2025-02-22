use std::env;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use chrono::{DateTime, Utc};

// Configuration
const WORKER_ID_BITS: u64 = 10;
const SEQUENCE_BITS: u64 = 12;
const DEFAULT_EPOCH: u64 = 1672531200000; // Default epoch

fn decode_snowflake(snowflake_id: u64, epoch: u64) -> (u64, u64, u64) {
    let worker_id_mask = (1 << WORKER_ID_BITS) - 1;
    let sequence_mask = (1 << SEQUENCE_BITS) - 1;

    let sequence = snowflake_id & sequence_mask;
    let worker_id = (snowflake_id >> SEQUENCE_BITS) & worker_id_mask;
    let timestamp = snowflake_id >> (WORKER_ID_BITS + SEQUENCE_BITS);

    let actual_timestamp = timestamp + epoch;

    (actual_timestamp, worker_id, sequence)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <snowflake_id> [epoch]", args[0]);
        return;
    }

    let snowflake_id: u64 = match args[1].parse() {
        Ok(id) => id,
        Err(_) => {
            eprintln!("Invalid Snowflake ID");
            return;
        }
    };

    let epoch: u64 = if args.len() >= 3 {
        match args[2].parse() {
            Ok(epoch) => epoch,
            Err(_) => {
                eprintln!("Invalid epoch, using default");
                DEFAULT_EPOCH
            }
        }
    } else {
        DEFAULT_EPOCH
    };

    let (timestamp, worker_id, sequence) = decode_snowflake(snowflake_id, epoch);

    let duration_since_epoch = Duration::from_millis(timestamp);
    let datetime: SystemTime = UNIX_EPOCH + duration_since_epoch;
    let datetime: DateTime<Utc> = DateTime::<Utc>::from(datetime);

    println!("Snowflake ID: {}", snowflake_id);
    println!("Timestamp (ms since epoch): {}", timestamp);
    println!("DateTime: {}", datetime.format("%Y-%m-%d %H:%M:%S%.3f UTC").to_string());
    println!("Worker ID: {}", worker_id);
    println!("Sequence: {}", sequence);
}