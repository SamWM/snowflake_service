use std::{env, io};
use std::io::{Read, Write};
use snowflake_generator::DEFAULT_EPOCH;
use snowflake_generator::generate_snowflake;
use snowflake_generator::get_worker_id;

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