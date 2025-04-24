use anyhow::{Context, Result};
use regex::Regex;
use scraper::{Html, Selector};
use lazy_static::lazy_static;
use serde::Serialize; // Import Serialize

pub const BASE_URL: &str = "https://www.nowinstock.net/computers/videocards/nvidia/";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";

lazy_static! {
    static ref PRICE_RE: Regex = Regex::new(r"[\d,]+\.\d{2}").unwrap();
}

#[derive(Debug, Clone, Serialize)] // Add Serialize derive
pub struct GpuListing {
    pub name: String,
    pub status: String,
    pub price: String,
    #[serde(skip_serializing_if = "Option::is_none")] // Don't include in JSON/YAML if None
    pub price_numeric: Option<f64>,
    pub last_available: String,
    pub link: String,
}

// Helper function to parse price string into a numeric value for sorting
fn parse_price(price_str: &str) -> Option<f64> {
    PRICE_RE.find(price_str).and_then(|mat| {
        mat.as_str().replace(',', "").parse::<f64>().ok()
    })
}

pub fn fetch_html(url: &str, quiet: bool) -> Result<String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .context("Failed to build reqwest client")?;

    if !quiet {
        println!("Downloading HTML from {}...", url);
    }
    let response = client.get(url)
        .send()
        .context(format!("Failed to fetch URL: {}", url))?;

    if !response.status().is_success() {
        anyhow::bail!("Request failed with status: {}", response.status());
    }

    let html_content = response.text().context("Failed to read response text")?;
    if !quiet {
        println!("Download complete.");
    }
    Ok(html_content)
}

pub fn parse_listings(html_content: &str, quiet: bool) -> Result<Vec<GpuListing>> {
     if !quiet {
        println!("Parsing HTML...");
     }
    let document = Html::parse_document(html_content);
    let table_selector = Selector::parse("#data > table.table").map_err(|e| anyhow::anyhow!("Invalid table selector: {}", e))?;
    let row_selector = Selector::parse("tbody > tr").map_err(|e| anyhow::anyhow!("Invalid row selector: {}", e))?;
    let cell_selector = Selector::parse("td").map_err(|e| anyhow::anyhow!("Invalid cell selector: {}", e))?;
    let link_selector = Selector::parse("a").map_err(|e| anyhow::anyhow!("Invalid link selector: {}", e))?;

    let mut listings = Vec::new();

    if let Some(table_element) = document.select(&table_selector).next() {
        for row_element in table_element.select(&row_selector) {
            let cells: Vec<_> = row_element.select(&cell_selector).collect();

            // Handle special cases like Ebay row
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
                        continue; // Skip to next row
                    }
                }
            }

            // Standard row parsing
            if cells.len() < 4 {
                continue; // Skip incomplete rows
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
                    // Prefer the title attribute for full timestamp if available
                    cell.value().attr("title").map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).unwrap_or(main_text)
                })
                .unwrap_or_else(|| "-".to_string());

            // Skip rows that couldn't be parsed properly
            if name == "N/A" || name.is_empty() {
                continue;
            }

            listings.push(GpuListing {
                name,
                status,
                price,
                price_numeric, // Keep numeric price for sorting
                last_available,
                link,
            });
        }
    } else {
        // Only bail if we are not in quiet mode, otherwise return empty list maybe?
        // Or perhaps always bail, as it indicates a site structure change.
        anyhow::bail!("Could not find the data table using selector '#data > table.table'. The page structure might have changed.");
    }

    if !quiet {
        println!("Parsing complete. Found {} listings.", listings.len());
        if listings.is_empty() {
            println!("Warning: No listings were successfully parsed. Check the URL and HTML structure.");
        }
    }
    Ok(listings)
}