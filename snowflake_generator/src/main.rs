use std::net::{IpAddr};
use std::sync::{Mutex};
use std::sync::atomic::AtomicU64;
use std::{env, io};
use std::io::{Read, Write};

// Configuration
const WORKER_ID_BITS: u64 = 10;
const SEQUENCE_BITS: u64 = 12;
const DEFAULT_EPOCH: u64 = 1672531200000;

// Shared state
static SEQUENCE: AtomicU64 = AtomicU64::new(0);
static LAST_TIMESTAMP: Mutex<u64> = Mutex::new(0);

fn get_worker_id() -> (u64, Option<IpAddr>) {
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

fn generate_snowflake(epoch: u64) -> u64 {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let worker_id = get_worker_id().0;

    let mut last_timestamp = LAST_TIMESTAMP.lock().unwrap();

    if timestamp < *last_timestamp {
        panic!("Clock moved backwards");
    }

    let sequence = if timestamp == *last_timestamp {
        SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::SeqCst) & ((1 << SEQUENCE_BITS) - 1)
    } else {
        SEQUENCE.store(0, std::sync::atomic::Ordering::SeqCst);
        0
    };

    *last_timestamp = timestamp;

    if sequence == 0 && timestamp == *last_timestamp + 1 {
        while std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
            <= *last_timestamp
        {}
        return generate_snowflake(epoch);
    }

    (timestamp - epoch) << (WORKER_ID_BITS + SEQUENCE_BITS)
        | (worker_id << SEQUENCE_BITS)
        | sequence
}

fn handle_connection(mut stream: std::net::TcpStream, epoch: u64) -> io::Result<()> {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;

    let request = std::str::from_utf8(&buffer).unwrap_or("");
    let mut response = String::new();

    if request.starts_with("GET /") || request.starts_with("GET /snowflake") {
        let mut count = 1;

        if let Some(count_str) = request.split("count=").nth(1) {
            if let Ok(parsed_count) = count_str.split_whitespace().next().unwrap_or("1").parse::<usize>() {
                count = parsed_count;
            }
        }

        response.push_str("HTTP/1.1 200 OK\r\n\r\n");
        for i in 0..count {
            response.push_str(&generate_snowflake(epoch).to_string());
            if i < count - 1 {
                response.push_str("\n");
            }
        }
    } else {
        response = "HTTP/1.1 404 Not Found\r\n\r\n".to_string();
    }

    stream.write(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let epoch: u64 = if args.len() > 1 {
        args[1].parse().unwrap_or(DEFAULT_EPOCH)
    } else {
        DEFAULT_EPOCH
    };

    if args.len() >= 2 && args[1] == "--generate" {
        let count = if args.len() >= 3 {
            args[2].parse().unwrap_or(1)
        } else {
            1
        };
        let gen_epoch = if args.len() >= 4 {
            args[3].parse().unwrap_or(DEFAULT_EPOCH)
        } else {
            DEFAULT_EPOCH
        };

        for _ in 0..count {
            println!("{}", generate_snowflake(gen_epoch));
        }
        return Ok(());
    }

    let port: u16 = if args.len() >= 3 {
        args[2].parse().unwrap_or(8080)
    } else {
        8080
    };

    let (worker_id, ip_addr) = get_worker_id();

    let address = format!("0.0.0.0:{}", port);
    let listener = std::net::TcpListener::bind(address)?;
    println!("Server listening on 0.0.0.0:{}", port);
    println!("Worker ID: {}", worker_id);

    if let Some(ip) = ip_addr {
        println!("IP Address used for Worker ID: {}", ip);
    }

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let epoch_clone = epoch;
                std::thread::spawn(move || {
                    if let Err(err) = handle_connection(stream, epoch_clone) {
                        eprintln!("Error handling connection: {}", err);
                    }
                });
            }
            Err(err) => {
                eprintln!("Error accepting connection: {}", err);
            }
        }
    }
    Ok(())
}