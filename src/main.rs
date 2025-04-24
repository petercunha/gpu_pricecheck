use anyhow::Result;
use clap::Parser;
use std::cmp::Ordering;

// Declare modules
mod cli;
mod output;
mod scraper;

// Use items from modules
use cli::{Args, OutputFormat, SortColumn};
use scraper::{GpuListing, BASE_URL};

fn main() -> Result<()> {
    let args = Args::parse();

    // Determine if logging should be suppressed
    let quiet = args.format != OutputFormat::Table;

    let url = format!("{}{}", BASE_URL, args.gpu);

    let html = scraper::fetch_html(&url, quiet)?;
    let mut listings = scraper::parse_listings(&html, quiet)?;

    // --- Filtering ---
    if !args.all {
        let original_count = listings.len();
        listings.retain(|item| {
            let lower_status = item.status.to_lowercase();
            // Keep "Stock Available" (Ebay) even when filtering
            lower_status != "out of stock" && lower_status != "not tracking"
        });
        let filtered_count = listings.len();
        if !quiet && original_count > filtered_count {
            println!(
                "Filtered out {} unavailable listings (Out of Stock, Not Tracking). Use --all to show.",
                original_count - filtered_count
            );
        }
    } else if !quiet {
        println!("Showing all listings (--all flag detected).");
    }

    // --- Sorting ---
    if !listings.is_empty() {
         if !quiet {
            println!("Sorting by {:?} {}...", args.sort_by, if args.desc { "descending" } else { "ascending" });
         }
        listings.sort_by(|a, b| {
            let ordering = match args.sort_by {
                SortColumn::Name => a.name.cmp(&b.name),
                SortColumn::Status => a.status.cmp(&b.status),
                SortColumn::Price => {
                    // Sort by numeric price first, then by string representation
                    match (a.price_numeric, b.price_numeric) {
                        (Some(pa), Some(pb)) => pa.partial_cmp(&pb).unwrap_or(Ordering::Equal),
                        (Some(_), None) => Ordering::Less, // Items with price come first
                        (None, Some(_)) => Ordering::Greater, // Items without price come last
                        (None, None) => a.price.cmp(&b.price), // Sort by text if both lack numeric
                    }
                },
                SortColumn::LastAvailable => a.last_available.cmp(&b.last_available), // Simple string compare for now
                SortColumn::Link => a.link.cmp(&b.link),
            };

            if args.desc {
                ordering.reverse()
            } else {
                ordering
            }
        });
    }

    // --- Limiting ---
    if let Some(limit) = args.limit {
        if limit < listings.len() {
             if !quiet {
                println!("Limiting results to the top {} listings.", limit);
             }
            listings.truncate(limit);
        }
    }

    // --- Output ---
    match args.format {
        OutputFormat::Table => output::print_table(&listings, &args.sort_by, args.desc),
        OutputFormat::Json => output::print_json(&listings)?,
        OutputFormat::Yaml => output::print_yaml(&listings)?,
        OutputFormat::Toml => output::print_toml(&listings)?,
    }

    Ok(())
}