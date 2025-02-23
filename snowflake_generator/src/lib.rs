use std::net::{IpAddr};
use std::sync::{Mutex, OnceLock};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

// Configuration
const WORKER_ID_BITS: u64 = 10;
const SEQUENCE_BITS: u64 = 12;
const MAX_SEQUENCE: u64 = (1 << SEQUENCE_BITS) - 1;
pub const DEFAULT_EPOCH: u64 = 1672531200000;

// Shared state
static SEQUENCE: AtomicU64 = AtomicU64::new(0);
static LAST_TIMESTAMP: Mutex<u64> = Mutex::new(0);

static WORKER_ID_CACHE: OnceLock<(u64, Option<IpAddr>)> = OnceLock::new();

pub fn get_worker_id() -> (u64, Option<IpAddr>) {
    *WORKER_ID_CACHE.get_or_init(get_worker_id_once)
}

fn get_worker_id_once() -> (u64, Option<IpAddr>) {
    match local_ip_address::local_ip() {
        Ok(ip) => {
            match ip {
                IpAddr::V4(ipv4) => {
                    let octets = ipv4.octets();
                    let worker_id = ((octets[2] as u64) << 8) | (octets[3] as u64);
                    (worker_id & ((1 << WORKER_ID_BITS) - 1), Some(IpAddr::V4(ipv4)))
                }
                IpAddr::V6(ipv6) => {
                    (0, Some(IpAddr::V6(ipv6)))
                }
            }
        }
        Err(e) => {
            eprintln!("Error getting local IP: {}", e);
            (0, None)
        }
    }
}

pub fn generate_snowflake(epoch: u64) -> u64 {
    loop {
        let mut last_timestamp_guard = LAST_TIMESTAMP.lock().unwrap();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        if timestamp < *last_timestamp_guard {
            panic!("Clock moved backwards");
        }

        if timestamp != *last_timestamp_guard {
            *last_timestamp_guard = timestamp;
            SEQUENCE.store(0, Ordering::SeqCst);
        }

        let sequence = SEQUENCE.load(Ordering::SeqCst);
        if sequence > MAX_SEQUENCE {
            let saved_timestamp = timestamp;
            drop(last_timestamp_guard);
            while SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64 == saved_timestamp {}
            continue;
        }

        let next_sequence = SEQUENCE.fetch_add(1, Ordering::SeqCst);

        if next_sequence > MAX_SEQUENCE {
          continue;
        }

        return (timestamp - epoch) << (WORKER_ID_BITS + SEQUENCE_BITS)
            | (get_worker_id().0 << SEQUENCE_BITS)
            | next_sequence;
    }
}