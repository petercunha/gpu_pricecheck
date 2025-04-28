use clap::{Parser, ValueEnum};
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum GpuModel {
    #[value(name = "5090")]
    Rtx5090,
    #[value(name = "5080")]
    Rtx5080,
    #[value(name = "5070ti")]
    Rtx5070Ti,
    #[value(name = "5070")]
    Rtx5070,
}

// Implement Display to get the URL part
impl fmt::Display for GpuModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GpuModel::Rtx5090 => write!(f, "rtx5090/"),
            GpuModel::Rtx5080 => write!(f, "rtx5080/"),
            GpuModel::Rtx5070Ti => write!(f, "rtx5070ti/"),
            GpuModel::Rtx5070 => write!(f, "rtx5070/"),
        }
    }
}

#[derive(Debug, Error)]
#[error("Invalid GPU model: {0}")]
pub struct ParseGpuModelError(String);

impl FromStr for GpuModel {
    type Err = ParseGpuModelError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim_end_matches('/') {
            "5090" | "rtx5090" => Ok(GpuModel::Rtx5090),
            "5080" | "rtx5080" => Ok(GpuModel::Rtx5080),
            "5070ti" | "rtx5070ti" => Ok(GpuModel::Rtx5070Ti),
            "5070" | "rtx5070" => Ok(GpuModel::Rtx5070),
            _ => Err(ParseGpuModelError(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum SortColumn {
    Name,
    Status,
    Price,
    #[value(name="last")]
    LastAvailable,
    Link,
}

// Allow parsing from string for clap
impl std::str::FromStr for SortColumn {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "name" => Ok(SortColumn::Name),
            "status" => Ok(SortColumn::Status),
            "price" => Ok(SortColumn::Price),
            "last" | "lastavailable" | "last_available" => Ok(SortColumn::LastAvailable),
            "link" => Ok(SortColumn::Link),
            _ => Err(format!("Invalid sort column: {}", s)),
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Yaml,
    Toml,
}

// Helper function to parse SocketAddr
fn parse_socket_addr(s: &str) -> Result<SocketAddr, String> {
    // Try parsing as full SocketAddr first
    if let Ok(addr) = SocketAddr::from_str(s) {
        return Ok(addr);
    }
    // Try parsing as just a port number
    if let Ok(port) = u16::from_str(s) {
        // Default to localhost if only port is given
        return Ok(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port));
    }
    Err(format!("Invalid socket address or port: {}", s))
}

#[derive(Parser, Debug)]
#[command(author, version, about = "Checks GPU stock and prices from nowinstock.net", long_about = None)]
pub struct Args {
    /// GPU Model to check stock for (ignored if --cheapest-each or --web is used)
    #[arg(value_enum, default_value = "5080")]
    pub gpu: GpuModel,

    /// Column to sort by (used by CLI, potentially web in future)
    #[arg(short, long, value_enum, default_value = "price")]
    pub sort_by: SortColumn,

    /// Sort in descending order (used by CLI, potentially web in future)
    #[arg(short, long)]
    pub desc: bool,

    /// Show all listings, including Out of Stock/Not Tracking (used by CLI, potentially web in future)
    #[arg(long)]
    pub all: bool,

    /// Limit the number of results shown (used by CLI, potentially web in future)
    #[arg(short = 'n', long, value_parser = clap::value_parser!(usize))]
    pub limit: Option<usize>,

    /// Output format (used by CLI)
    #[arg(short, long, value_enum, default_value = "table")]
    pub format: OutputFormat,

    /// Find the single cheapest available listing for each GPU model (used by CLI)
    #[arg(short = 'c', long)]
    pub cheapest_each: bool,

    /// Run as a web server instead of a one-off CLI command
    #[arg(short = 'w', long)]
    pub web: bool,

    /// Port and optional address for the web server (e.g., 8080 or 0.0.0.0:8080)
    #[arg(long, value_parser = parse_socket_addr, default_value = "127.0.0.1:8080")]
    pub listen: SocketAddr,

    /// Enable verbose logging output (default is minimal logging)
    #[arg(short, long)]
    pub verbose: bool,
}