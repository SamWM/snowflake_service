# Snowflake ID Generator and Decoder

This project contains two Rust binaries: `snowflake_generator` and `snowflake_decoder`, designed to generate and decode Snowflake IDs.

## Tool Requirements

Before building and running the project, ensure you have the following tools installed:

* **Rust and Cargo:** You'll need the Rust toolchain, including the Rust compiler (`rustc`) and Cargo (the Rust package manager). Install Rust from [rustup.rs](https://rustup.rs/).
* **A C/C++ Compiler (for `local-ip-address`):** The `local-ip-address` crate, used by `snowflake_generator` to derive a worker ID, requires a C/C++ compiler.
    * On Windows, you can install the "Build Tools for Visual Studio 2022" with the "Desktop development with C++" workload. Chocolatey or Scoop.sh are also viable options.
    * On Linux, you'll need `gcc` or `clang`.
    * On macOS, you'll need Xcode or Command Line Tools for Xcode.
* **Git (Optional):** If you're cloning the repository from a version control system, you'll need Git.

## Building the Project

1.  **Clone the Repository (if applicable):**

    ```bash
    git clone <repository_url>
    cd <project_directory>
    ```

2.  **Build the Binaries:**

    ```bash
    cargo build --release
    ```

    This command will build both `snowflake_generator` and `snowflake_decoder` binaries in the `target/release` directory.

## Usage

### `snowflake_generator`

This binary generates Snowflake IDs.

**Usage:**

```bash
./target/release/snowflake_generator [epoch] [port] [--generate] [count] [gen_epoch]
```

* `[epoch]` (Optional): The epoch in milliseconds since the Unix epoch. Defaults to `1672531200000`.
* `[port]` (Optional): The port to listen on for the service. Defaults to `8080`.
* `--generate` (Optional): If provided, generates Snowflake IDs and exits.
    * `[count]` (Optional): The number of IDs to generate. Defaults to `1`.
    * `[gen_epoch]` (Optional): The epoch to use when generating. Defaults to the main epoch.

**Examples:**

* Run as a service on port 3000 with a custom epoch:

    ```
    ./target/release/snowflake_generator 1678848000000 3000
    ```

* Generate 5 Snowflake IDs with a custom epoch and exit:

    ```
    ./target/release/snowflake_generator --generate 5 1678848000000
    ```

* Generate 1 Snowflake ID with the default epoch and exit:

    ```
    ./target/release/snowflake_generator --generate
    ```

* Run as a service with the default epoch:

    ```
    ./target/release/snowflake_generator
    ```

### `snowflake_decoder`

This binary decodes Snowflake IDs.

**Usage:**

```
./target/release/snowflake_decoder <snowflake_id> [epoch]
```

* `<snowflake_id>`: The Snowflake ID to decode.
* `[epoch]` (Optional): The epoch used to generate the Snowflake ID. Defaults to `1672531200000`.

**Examples:**

* Decode a Snowflake ID with a custom epoch:

    ```
    ./target/release/snowflake_decoder 17565551988224 1678848000000
    ```

* Decode a Snowflake ID with the default epoch:

    ```
    ./target/release/snowflake_decoder 17565551988224
    ```

## Cargo Workspace

This project is organized as a Cargo workspace, allowing you to build both binaries from the root directory.

## Dependencies

* `local-ip-address`: Used by `snowflake_generator` to derive a worker ID from the local IP address.

## Notes

* Ensure that the provided epoch is correct for accurate decoding.
* The `snowflake_generator` service will log the worker ID and the IP address used to derive it when starting.
* For the `snowflake_generator` service to avoid UAC prompts on Windows, use a port greater than 1023.