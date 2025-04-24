use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, CellAlignment, Color, ContentArrangement,
    Table, Attribute, ColumnConstraint, Width
};
use regex::Regex;
use scraper::{Html, Selector};
use lazy_static::lazy_static;
use std::fmt;

const BASE_URL: &str = "https://www.nowinstock.net/computers/videocards/nvidia/";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";

lazy_static! {
    static ref PRICE_RE: Regex = Regex::new(r"[\d,]+\.\d{2}").unwrap();
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum SortColumn {
    Name,
    Status,
    Price,
    LastAvailable,
    Link,
}

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

// Enum for GPU Model selection
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum GpuModel {
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

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// GPU Model to check stock for
    #[arg(value_enum, default_value = "5080")]
    gpu: GpuModel,

    /// Column to sort by (name, status, price, last, link)
    #[arg(short, long, value_parser = clap::value_parser!(SortColumn), default_value = "price")]
    sort_by: SortColumn,

    /// Sort in descending order
    #[arg(short, long)]
    desc: bool,

    /// Show all listings, including Out of Stock/Not Tracking
    #[arg(long)]
    all: bool,

    /// Limit the number of results shown
    #[arg(short = 'n', long, value_parser = clap::value_parser!(usize))]
    limit: Option<usize>,
}

#[derive(Debug, Clone)]
struct GpuListing {
    name: String,
    status: String,
    price: String,
    price_numeric: Option<f64>,
    last_available: String,
    link: String,
}

// Helper function to parse price string into a numeric value for sorting
fn parse_price(price_str: &str) -> Option<f64> {
    PRICE_RE.find(price_str).and_then(|mat| {
        mat.as_str().replace(',', "").parse::<f64>().ok()
    })
}

fn fetch_html(url: &str) -> Result<String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .context("Failed to build reqwest client")?;

    println!("Downloading HTML from {}...", url);
    let response = client.get(url)
        .send()
        .context(format!("Failed to fetch URL: {}", url))?;

    if !response.status().is_success() {
        anyhow::bail!("Request failed with status: {}", response.status());
    }

    let html_content = response.text().context("Failed to read response text")?;
    println!("Download complete.");
    Ok(html_content)
}

fn parse_listings(html_content: &str) -> Result<Vec<GpuListing>> {
    println!("Parsing HTML...");
    let document = Html::parse_document(html_content);
    let table_selector = Selector::parse("#data > table.table").map_err(|e| anyhow::anyhow!("Invalid table selector: {}", e))?;
    let row_selector = Selector::parse("tbody > tr").map_err(|e| anyhow::anyhow!("Invalid row selector: {}", e))?;
    let cell_selector = Selector::parse("td").map_err(|e| anyhow::anyhow!("Invalid cell selector: {}", e))?;
    let link_selector = Selector::parse("a").map_err(|e| anyhow::anyhow!("Invalid link selector: {}", e))?;

    let mut listings = Vec::new();

    if let Some(table_element) = document.select(&table_selector).next() {
        for row_element in table_element.select(&row_selector) {
            let cells: Vec<_> = row_element.select(&cell_selector).collect();

            if cells.len() == 2 {
                if let Some(link_element) = cells[0].select(&link_selector).next() {
                    let name_text = link_element.text().collect::<String>().trim().to_string();
                    if name_text.contains("Ebay") {
                        let link = link_element.value().attr("href").unwrap_or("").to_string();
                        let status = cells[1].text().collect::<String>().trim().to_string();
                        listings.push(GpuListing {
                            name: name_text,
                            status,
                            price: "-".to_string(),
                            price_numeric: None,
                            last_available: "-".to_string(),
                            link,
                        });
                        continue;
                    }
                }
            }

            if cells.len() < 4 {
                continue;
            }

            let (name, link) = cells.get(0)
                .and_then(|cell| cell.select(&link_selector).next())
                .map(|link_el| {
                    let name_text = link_el.text().collect::<String>().trim().to_string();
                    let href = link_el.value().attr("href").unwrap_or("").to_string();
                    (name_text, href)
                })
                .unwrap_or_else(|| ("N/A".to_string(), "".to_string()));

            let status = cells.get(1)
                .map(|cell| {
                    cell.select(&link_selector).next()
                       .map(|link_el| link_el.text().collect::<String>())
                       .unwrap_or_else(|| cell.text().collect::<String>())
                       .trim()
                       .to_string()
                })
                .unwrap_or_else(|| "N/A".to_string());

            let price = cells.get(2)
                .map(|cell| cell.text().collect::<String>().trim().to_string())
                .unwrap_or_else(|| "-".to_string());
            let price_numeric = parse_price(&price);

            let last_available = cells.get(3)
                .map(|cell| {
                    let main_text = cell.text().collect::<String>().trim().to_string();
                    cell.value().attr("title").unwrap_or(&main_text).to_string()
                })
                .unwrap_or_else(|| "-".to_string());

            if name == "N/A" || name.is_empty() {
                continue;
            }

            listings.push(GpuListing {
                name,
                status,
                price,
                price_numeric,
                last_available,
                link,
            });
        }
    } else {
        anyhow::bail!("Could not find the data table using selector '#data > table.table'. The page structure might have changed.");
    }

    println!("Parsing complete. Found {} listings.", listings.len());
    if listings.is_empty() {
        println!("Warning: No listings were successfully parsed. Check the URL and HTML structure.");
    }
    Ok(listings)
}

fn create_status_cell(status: &str) -> Cell {
    let cell = Cell::new(status).set_alignment(CellAlignment::Center);
    match status.to_lowercase().as_str() {
        "in stock" => cell.add_attribute(Attribute::Bold).fg(Color::Green),
        "preorder" => cell.add_attribute(Attribute::Bold).fg(Color::Yellow),
        "out of stock" | "not tracking" => cell.fg(Color::Red),
        "stock available" => cell.fg(Color::DarkGreen),
        _ => cell,
    }
}

// Helper function to create a clickable link for terminals supporting it
// fn create_clickable_link(url: &str, text: &str) -> String {
//     if url.is_empty() {
//         text.to_string()
//     } else {
//         // Use BEL (\x07) as the terminator instead of ESC \ (\x1B\\)
//         format!("\x1B]8;;{}\x07{}\x1B]8;;\x07", url, text)
//     }
// }

fn print_table(listings: &[GpuListing], sort_by: &SortColumn, descending: bool) {
    if listings.is_empty() {
        println!("No listings found to display (after filtering).");
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic); // Keep dynamic arrangement

    // Adjust constraints - give Link a minimum width too
    table.set_constraints(vec![
        ColumnConstraint::LowerBoundary(Width::Fixed(70)), // Name
        ColumnConstraint::LowerBoundary(Width::Fixed(15)), // Status
        ColumnConstraint::LowerBoundary(Width::Fixed(15)), // Price
        ColumnConstraint::LowerBoundary(Width::Fixed(35)), // Last Available
        ColumnConstraint::LowerBoundary(Width::Fixed(25)), // Link - Increase minimum width
    ]);

    table.set_header(vec![
        Cell::new(format!("Name {}", if *sort_by == SortColumn::Name { if descending {"▼"} else {"▲"}} else {""})).add_attribute(Attribute::Bold),
        Cell::new(format!("Status {}", if *sort_by == SortColumn::Status { if descending {"▼"} else {"▲"}} else {""})).add_attribute(Attribute::Bold).set_alignment(CellAlignment::Center),
        Cell::new(format!("Price {}", if *sort_by == SortColumn::Price { if descending {"▼"} else {"▲"}} else {""})).add_attribute(Attribute::Bold).set_alignment(CellAlignment::Right),
        Cell::new(format!("Last Available {}", if *sort_by == SortColumn::LastAvailable { if descending {"▼"} else {"▲"}} else {""})).add_attribute(Attribute::Bold).set_alignment(CellAlignment::Right),
        Cell::new(format!("Link {}", if *sort_by == SortColumn::Link { if descending {"▼"} else {"▲"}} else {""})).add_attribute(Attribute::Bold), // Keep header simple
    ]);

    for item in listings {
        table.add_row(vec![
            Cell::new(&item.name),
            create_status_cell(&item.status),
            Cell::new(&item.price).set_alignment(CellAlignment::Right),
            Cell::new(&item.last_available).set_alignment(CellAlignment::Right),
            Cell::new(&item.link), // Display the raw URL directly
        ]);
    }

    println!("{}", table);
}

fn main() -> Result<()> {
    let args = Args::parse();

    let url = format!("{}{}", BASE_URL, args.gpu);

    let html = fetch_html(&url)?;
    let mut listings = parse_listings(&html)?;

    if !args.all {
        let original_count = listings.len();
        listings.retain(|item| {
            let lower_status = item.status.to_lowercase();
            lower_status != "out of stock" && lower_status != "not tracking"
        });
        let filtered_count = listings.len();
        if original_count > filtered_count {
            println!("Filtered out {} unavailable listings (Out of Stock, Not Tracking). Use --all to show.", original_count - filtered_count);
        }
    } else {
        println!("Showing all listings (--all flag detected).");
    }

    if !listings.is_empty() {
        println!("Sorting by {:?} {}...", args.sort_by, if args.desc { "descending" } else { "ascending" });
        listings.sort_by(|a, b| {
            let ordering = match args.sort_by {
                SortColumn::Name => a.name.cmp(&b.name),
                SortColumn::Status => a.status.cmp(&b.status),
                SortColumn::Price => {
                    match (a.price_numeric, b.price_numeric) {
                        (Some(pa), Some(pb)) => pa.partial_cmp(&pb).unwrap_or(std::cmp::Ordering::Equal),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => a.price.cmp(&b.price),
                    }
                },
                SortColumn::LastAvailable => a.last_available.cmp(&b.last_available),
                SortColumn::Link => a.link.cmp(&b.link),
            };

            if args.desc {
                ordering.reverse()
            } else {
                ordering
            }
        });
    }

    if let Some(limit) = args.limit {
        if limit < listings.len() {
            println!("Limiting results to the top {} listings.", limit);
            listings.truncate(limit);
        }
    }

    print_table(&listings, &args.sort_by, args.desc);

    Ok(())
}