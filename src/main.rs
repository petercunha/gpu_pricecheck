use anyhow::{Context, Result}; // Add Context
use clap::{Parser, ValueEnum}; // Add ValueEnum
use std::cmp::Ordering;

// Declare modules
mod cli;
mod output;
mod scraper;

// Use items from modules
use cli::{Args, GpuModel, OutputFormat, SortColumn}; // Add GpuModel
use scraper::{GpuListing, BASE_URL}; // Add GpuListing

fn main() -> Result<()> {
    let args = Args::parse();

    // Determine if logging should be suppressed
    let quiet = args.format != OutputFormat::Table;
    let mut final_listings: Vec<GpuListing> = Vec::new();

    if args.cheapest_each {
        // --- Mode: Find cheapest for each model ---
        if !quiet {
            println!("Finding the cheapest available listing for each GPU model...");
        }

        for model in GpuModel::value_variants() {
            let model_url = format!("{}{}", BASE_URL, model);
            if !quiet {
                println!("--- Checking: {:?} ---", model);
            }

            // Use a closure to handle errors gracefully for each model
            let cheapest_result = (|| -> Result<Option<GpuListing>> {
                let html = scraper::fetch_html(&model_url, quiet)
                    .with_context(|| format!("Failed to fetch HTML for {:?}", model))?;
                let mut listings = scraper::parse_listings(&html, quiet)
                    .with_context(|| format!("Failed to parse listings for {:?}", model))?;

                // Filter out unavailable listings (standard filtering)
                if !args.all {
                     listings.retain(|item| {
                        let lower_status = item.status.to_lowercase();
                        lower_status != "out of stock" && lower_status != "not tracking"
                    });
                }

                // Find the cheapest listing *with* a numeric price
                let cheapest = listings.into_iter()
                    .filter(|item| item.price_numeric.is_some()) // Only consider items with a price
                    .min_by(|a, b| {
                        // Unwraps are safe due to the filter above
                        a.price_numeric.unwrap().partial_cmp(&b.price_numeric.unwrap())
                         .unwrap_or(Ordering::Equal) // Handle potential NaN comparison if necessary
                    });

                Ok(cheapest)
            })(); // Immediately call the closure

            match cheapest_result {
                Ok(Some(listing)) => {
                    final_listings.push(listing);
                }
                Ok(None) => {
                    if !quiet {
                        println!("No available listing with a valid price found for {:?}.", model);
                    }
                }
                Err(e) => {
                    // Print error only if not quiet, but always log it potentially?
                    if !quiet {
                         eprintln!("Warning: Failed to process model {:?}: {:?}", model, e);
                    }
                    // Continue to the next model even if one fails
                }
            }
        }
         if !quiet {
             println!("--- Finished checking all models ---");
         }

    } else {
        // --- Mode: Original logic for a single GPU model ---
        let url = format!("{}{}", BASE_URL, args.gpu);
        let html = scraper::fetch_html(&url, quiet)?;
        let mut listings = scraper::parse_listings(&html, quiet)?;

        // --- Filtering ---
        if !args.all {
            let original_count = listings.len();
            listings.retain(|item| {
                let lower_status = item.status.to_lowercase();
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
        final_listings = listings; // Assign to the final list
    }

    // --- Sorting (Applied to results from either mode) ---
    if !final_listings.is_empty() {
         if !quiet {
            println!("Sorting results by {:?} {}...", args.sort_by, if args.desc { "descending" } else { "ascending" });
         }
        final_listings.sort_by(|a, b| {
            let ordering = match args.sort_by {
                SortColumn::Name => a.name.cmp(&b.name),
                SortColumn::Status => a.status.cmp(&b.status),
                SortColumn::Price => {
                    match (a.price_numeric, b.price_numeric) {
                        (Some(pa), Some(pb)) => pa.partial_cmp(&pb).unwrap_or(Ordering::Equal),
                        (Some(_), None) => Ordering::Less,
                        (None, Some(_)) => Ordering::Greater,
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

    // --- Limiting (Applied to results from either mode) ---
    if let Some(limit) = args.limit {
        if limit < final_listings.len() {
             if !quiet {
                println!("Limiting results to the top {} listings.", limit);
             }
            final_listings.truncate(limit);
        }
    }

    // --- Output ---
    match args.format {
        OutputFormat::Table => output::print_table(&final_listings, &args.sort_by, args.desc),
        OutputFormat::Json => output::print_json(&final_listings)?,
        OutputFormat::Yaml => output::print_yaml(&final_listings)?,
        OutputFormat::Toml => output::print_toml(&final_listings)?,
    }

    Ok(())
}