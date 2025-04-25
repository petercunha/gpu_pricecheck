# gpu_pricecheck

A simple Rust CLI tool and web server to check GPU stock and prices from nowinstock.net.

## Features

- Fetch current stock status and price for specific Nvidia GPU models.
- Filter listings (e.g., show only in-stock items).
- Sort listings by various columns (name, status, price, last available).
- Output results in different formats (Table, JSON, YAML, TOML).
- Find the single cheapest available listing across all tracked GPU models.
- Run as a persistent web server to view listings in a browser.

## Installation

1.  **Install Rust:** If you don't have Rust installed, follow the instructions at [rustup.rs](https://rustup.rs/).
2.  **Clone the repository:**
    ```sh
    git clone <repository-url>
    cd gpu_pricecheck
    ```
3.  **Build the project:**
    ```sh
    cargo build --release
    ```
    The executable will be located at `./target/release/gpu_pricecheck`.

## Usage

Run the tool from the command line for a one-off check:

```sh
# Using cargo run during development
cargo run -- [OPTIONS] [GPU]

# Using the compiled binary (e.g., after `cargo build --release`)
./target/release/gpu_pricecheck [OPTIONS] [GPU]
```

Or, run it as a web server:

```sh
# Run web server on default port 8080
cargo run -- --web

# Run web server on a specific port and allow external access
cargo run -- --web --listen 0.0.0.0:9000

# Using the compiled binary
./target/release/gpu_pricecheck --web --listen 8081
```

### Arguments

- `[GPU]`: GPU model to check stock for. Default is `5080`. Ignored if `--cheapest-each` or `--web` is used.
  Possible values: `5090`, `5080`, `5070ti`, `5070`.

### Options

**CLI Options (ignored when `--web` is used, except for `--help` and `--version`):**

- `-s, --sort-by <SORT_BY>`: Column to sort by. Default is `price`.
  Possible values: `name`, `status`, `price`, `last`, `link`.
- `-d, --desc`: Sort in descending order. Default is ascending.
- `--all`: Show all listings, including "Out of Stock" and "Not Tracking".
- `-n, --limit <LIMIT>`: Limit the number of results shown.
- `-f, --format <FORMAT>`: Output format. Default is `table`.
  Possible values: `table`, `json`, `yaml`, `toml`.
- `--cheapest-each`: Find and display the single cheapest available listing for each GPU model (5090, 5080, etc.). Ignores the `[GPU]` argument.

**Web Server Options:**

- `-w, --web`: Run as a web server instead of a one-off CLI command.
- `--listen <ADDRESS:PORT>`: The socket address (IP and port) for the web server to listen on. Default is `127.0.0.1:8080`. Examples: `8080`, `0.0.0.0:9000`.

**General Options:**

- `-h, --help`: Display help information.
- `-V, --version`: Display version information.

### Examples

**CLI Examples:**

1.  Check stock for the RTX 5090 and sort by price in descending order:
    ```sh
    cargo run -- 5090 -s price -d
    ```
2.  Show all listings for the RTX 5080, including unavailable ones, in JSON format:
    ```sh
    cargo run -- 5080 --all --format json
    ```
3.  Limit results to the top 5 cheapest listings for the 5070 Ti:
    ```sh
    cargo run -- 5070ti -n 5
    ```
4.  Find the cheapest available listing for each GPU model and output as YAML:
    ```sh
    cargo run -- --cheapest-each --format yaml
    ```

**Web Server Examples:**

1.  Start the web server on the default address `127.0.0.1:8080`:
    ```sh
    cargo run -- --web
    # Access in browser: http://127.0.0.1:8080
    ```
2.  Start the web server on port 9000, accessible from other machines on the network:
    ```sh
    cargo run -- --web --listen 0.0.0.0:9000
    # Access locally: http://127.0.0.1:9000
    # Access from other machines: http://<your-machine-ip>:9000
    ```

## Development

- **Formatting:** Uses `rustfmt` (standard Rust formatting).
  ```sh
  cargo fmt
  ```
- **Linting:** Uses `clippy` (standard Rust linter).
  ```sh
  cargo clippy --all-targets --all-features -- -D warnings
  ```

### Dependencies

This project uses the following Rust crates:

- [`reqwest`](https://crates.io/crates/reqwest): For HTTP requests.
- [`scraper`](https://crates.io/crates/scraper): For parsing HTML.
- [`comfy-table`](https://crates.io/crates/comfy-table): For displaying tabular data (CLI).
- [`anyhow`](https://crates.io/crates/anyhow): For error handling.
- [`clap`](https://crates.io/crates/clap): For command-line argument parsing.
- [`regex`](https://crates.io/crates/regex): For parsing price strings.
- [`lazy_static`](https://crates.io/crates/lazy_static): For static regex initialization.
- [`serde`](https://crates.io/crates/serde), [`serde_json`](https://crates.io/crates/serde_json), [`serde_yaml`](https://crates.io/crates/serde_yaml), [`toml`](https://crates.io/crates/toml): For serialization (JSON, YAML, TOML output).
- [`tokio`](https://crates.io/crates/tokio): Asynchronous runtime.
- [`axum`](https://crates.io/crates/axum): Web framework.
- [`askama`](https://crates.io/crates/askama), [`askama_axum`](https://crates.io/crates/askama_axum): HTML templating engine.
- [`tower-http`](https://crates.io/crates/tower-http): HTTP utility types and services (e.g., for static files).
- [`chrono`](https://crates.io/crates/chrono): For displaying timestamps in the web UI. (Implicit dependency via askama example, good to list)

## License

This project is licensed under the MIT License - see the LICENSE file for details (if one exists).
