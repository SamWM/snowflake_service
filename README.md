# Snowflake ID generator

Build using:
`cargo build --release`

Generate 5 snowflake IDs with a custom epoch:
```
./target/release/snowflake_service --generate 5 1678848000000
```

Generate 1 snowflake ID with the default epoch:

```
./target/release/snowflake_service --generate
```

Generate 3 snowflake IDs with the default epoch:

```
./target/release/snowflake_service --generate 3
```

Run as a service with a custom epoch:

```
./target/release/snowflake_service 1678848000000 8080
```

Run as a service with the default epoch:

```
./target/release/snowflake_service
```