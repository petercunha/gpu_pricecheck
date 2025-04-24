# GPU Price Checker

GPU Price Checker is a command-line tool that fetches and displays GPU stock and pricing information from [NowInStock.net](https://www.nowinstock.net). It allows users to filter, sort, and limit the results for easier analysis.

## Features

- Fetches GPU stock and pricing data from NowInStock.net.
- Supports filtering out unavailable listings (e.g., "Out of Stock", "Not Tracking").
- Allows sorting by various columns: Name, Status, Price, Last Available, and Link.
- Supports ascending or descending sort order.
- Limits the number of results displayed.
- Displays results in a clean, tabular format.

## Installation

1. Ensure you have [Rust](https://www.rust-lang.org/) installed on your system.
2. Clone this repository:
   ```sh
   git clone https://github.com/yourusername/gpu_pricecheck.git
   cd gpu_pricecheck
   ```
3. Install:
   ```sh
   cargo install --path .
   ```
   
## Usage

Run the tool from the command line:

```sh
gpu_pricecheck [GPU] [OPTIONS]
```

### Arguments

- `[GPU]`: GPU model to check stock for. Default is `5080`.  
  Possible values: `5090`, `5080`, `5070ti`, `5070`.

### Options

- `-s, --sort-by <SORT_BY>`: Column to sort by. Default is `price`.  
  Possible values: `name`, `status`, `price`, `last`, `link`.

- `-d, --desc`: Sort in descending order. Default is ascending.

- `--all`: Show all listings, including "Out of Stock" and "Not Tracking".

- `-n, --limit <LIMIT>`: Limit the number of results shown.

- `-h, --help`: Display help information.

- `-V, --version`: Display version information.

### Examples

1. Check stock for the RTX 5090 and sort by price in descending order:
   ```sh
   gpu_pricecheck.exe 5090 -s price -d
   ```

2. Show all listings for the RTX 5080, including unavailable ones:
   ```sh
   gpu_pricecheck.exe 5080 --all
   ```

3. Limit results to the top 5 cheapest listings:
   ```sh
   gpu_pricecheck.exe 5070 -n 5
   ```

## Development

To run the project in development mode:

```sh
cargo run -- [OPTIONS] [GPU]
```

### Dependencies

This project uses the following Rust crates:

- [`reqwest`](https://crates.io/crates/reqwest): For HTTP requests.
- [`scraper`](https://crates.io/crates/scraper): For parsing HTML.
- [`comfy-table`](https://crates.io/crates/comfy-table): For displaying tabular data.
- [`anyhow`](https://crates.io/crates/anyhow): For error handling.
- [`clap`](https://crates.io/crates/clap): For command-line argument parsing.
- [`regex`](https://crates.io/crates/regex): For parsing price strings.
- [`lazy_static`](https://crates.io/crates/lazy_static): For static regex initialization.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Data is fetched from [NowInStock.net](https://www.nowinstock.net).
