use clap::{Parser, ValueEnum};
use std::fmt;

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


#[derive(Parser, Debug)]
#[command(author, version, about = "Checks GPU stock and prices from nowinstock.net", long_about = None)]
pub struct Args {
    /// GPU Model to check stock for
    #[arg(value_enum, default_value = "5080")]
    pub gpu: GpuModel,

    /// Column to sort by
    #[arg(short, long, value_enum, default_value = "price")]
    pub sort_by: SortColumn,

    /// Sort in descending order
    #[arg(short, long)]
    pub desc: bool,

    /// Show all listings, including Out of Stock/Not Tracking
    #[arg(long)]
    pub all: bool,

    /// Limit the number of results shown
    #[arg(short = 'n', long, value_parser = clap::value_parser!(usize))]
    pub limit: Option<usize>,

    /// Output format
    #[arg(short, long, value_enum, default_value = "table")]
    pub format: OutputFormat,
}