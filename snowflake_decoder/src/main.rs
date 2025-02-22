use std::env;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::str;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Utc};
use serde_json::{json, Value};

// Configuration
const WORKER_ID_BITS: u64 = 10;
const SEQUENCE_BITS: u64 = 12;
const DEFAULT_EPOCH: u64 = 1672531200000;

fn decode_snowflake(snowflake_id: u64, epoch: u64) -> (u64, u64, u64) {
    let worker_id_mask = (1 << WORKER_ID_BITS) - 1;
    let sequence_mask = (1 << SEQUENCE_BITS) - 1;

    let sequence = snowflake_id & sequence_mask;
    let worker_id = (snowflake_id >> SEQUENCE_BITS) & worker_id_mask;
    let timestamp = snowflake_id >> (WORKER_ID_BITS + SEQUENCE_BITS);

    let actual_timestamp = timestamp + epoch;

    (actual_timestamp, worker_id, sequence)
}

fn decode_snowflake_and_format(snowflake_id: u64, epoch: u64) -> Result<Value, Value> {
    // Extract timestamp part
    let timestamp_part = snowflake_id >> (WORKER_ID_BITS + SEQUENCE_BITS);

    // Calculate the actual timestamp in milliseconds since epoch
    let actual_timestamp_ms = timestamp_part + epoch;

    // Define a safe maximum timestamp (adjust as needed)
    let safe_max_timestamp_ms = 253402300800000; // Represents year 9999-12-31 23:59:59 UTC

    if actual_timestamp_ms > safe_max_timestamp_ms {
        return Err(json!({"error": "Snowflake ID timestamp is too large, potential overflow"}));
    }

    let (timestamp, worker_id, sequence) = decode_snowflake(snowflake_id, epoch);

    let duration_since_epoch = Duration::from_millis(timestamp);
    let datetime: SystemTime = UNIX_EPOCH + duration_since_epoch;
    let datetime: DateTime<Utc> = DateTime::<Utc>::from(datetime);

    Ok(json!({
        "snowflake_id": snowflake_id,
        "timestamp": timestamp,
        "datetime": datetime.format("%Y-%m-%d %H:%M:%S%.3f UTC").to_string(),
        "worker_id": worker_id,
        "sequence": sequence,
    }))
}

fn handle_connection(mut stream: TcpStream, epoch: u64) -> io::Result<()> {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;

    let request = str::from_utf8(&buffer).unwrap_or("");
    let mut response = String::new();

    if request.starts_with("GET /?id=") {
        if let Some(id_str) = request.split("id=").nth(1) {
            if let Some(id_str) = id_str.split_whitespace().next() {
                if let Ok(snowflake_id) = id_str.parse::<u64>() {
                    match decode_snowflake_and_format(snowflake_id, epoch) {
                        Ok(json_response) => {
                            response.push_str("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n");
                            response.push_str(&json_response.to_string());
                        }
                        Err(error_json) => {
                            response.push_str("HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\n\r\n");
                            response.push_str(&error_json.to_string());
                        }
                    }
                } else {
                    response.push_str("HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\n\r\n");
                    response.push_str(&json!({"error": "Invalid Snowflake ID"}).to_string());
                }
            }
        } else {
            response.push_str("HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\n\r\n");
            response.push_str(&json!({"error": "Missing id parameter"}).to_string());
        }
    } else {
        response.push_str("HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\n\r\n");
        response.push_str(&json!({"error": "Not Found"}).to_string());
    }

    stream.write(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() >= 2 && args[1] == "--decode" {
        if args.len() < 3 {
            eprintln!("Usage: {} --decode <snowflake_id> [epoch]", args[0]);
            return Ok(());
        }

        let snowflake_id: u64 = match args[2].parse() {
            Ok(id) => id,
            Err(_) => {
                eprintln!("Invalid Snowflake ID");
                return Ok(());
            }
        };

        let epoch: u64 = if args.len() >= 4 {
            match args[3].parse() {
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

        let json_response = json!({
            "snowflake_id": snowflake_id,
            "timestamp": timestamp,
            "datetime": datetime.format("%Y-%m-%d %H:%M:%S%.3f UTC").to_string(),
            "worker_id": worker_id,
            "sequence": sequence,
        });

        println!("{}", json_response.to_string());
        return Ok(());
    }

    let epoch: u64 = if args.len() > 1 {
        match args[1].parse() {
            Ok(epoch) => epoch,
            Err(_) => {
                eprintln!("Invalid epoch, using default");
                DEFAULT_EPOCH
            }
        }
    } else {
        DEFAULT_EPOCH
    };

    let port: u16 = if args.len() > 2 {
        match args[2].parse() {
            Ok(port) => port,
            Err(_) => {
                eprintln!("Invalid port, using default");
                8081
            }
        }
    } else {
        8081
    };

    let address = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(address)?;
    println!("Decoder service listening on 0.0.0.0:{}", port);

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