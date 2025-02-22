FROM rust:alpine

WORKDIR /app

COPY . .

RUN cargo build --release

EXPOSE 8080

CMD ["./target/release/snowflake_service", "1672531200000"] #default epoch