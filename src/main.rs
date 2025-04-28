use anyhow::{Context, Result};
use clap::{Parser, ValueEnum}; // Import ValueEnum trait

// Declare modules
mod cli;
mod output;
mod scraper;
mod web; // Add web module

// Use items from modules
use cli::{Args, GpuModel, OutputFormat, SortColumn};
use scraper::GpuListing; // Keep GpuListing import

#[tokio::main]
async fn main() -> Result<()> {
    // Make Args mutable so we can override defaults when no extra parameters are given.
    let mut args = Args::parse();
    // If only the program name is provided, set cheapest_each to true.
    if std::env::args().len() == 1 {
        args.cheapest_each = true;
    }
    if args.web {
        web::run_server(args.listen).await?;
    } else {
        run_cli(args).await?;
    }
    Ok(())
}

async fn run_cli(args: Args) -> Result<()> {
    let quiet = args.format != OutputFormat::Table;
    let mut final_listings: Vec<GpuListing> = Vec::new();

    if args.cheapest_each {
        if !quiet {
            println!("Finding the cheapest available listing for each GPU model...");
        }
        let models = GpuModel::value_variants();
        // Prepare a future for each model in parallel.
        let cheapest_futures = models.iter().map(|model| {
            let model = *model;
            async move {
                let model_url = format!("{}{}", scraper::BASE_URL, model);
                let res = (|| async {
                    let html = web::fetch_html(&model_url, quiet)
                        .await
                        .with_context(|| format!("Failed to fetch HTML for {:?}", model))?;
                    let mut listings = scraper::parse_listings(&html, quiet)
                        .with_context(|| format!("Failed to parse listings for {:?}", model))?;
                    if !args.all {
                        listings.retain(|item| {
                            let lower_status = item.status.to_lowercase();
                            lower_status != "out of stock" && lower_status != "not tracking"
                        });
                    }
                    let cheapest = listings.into_iter()
                        .filter(|item| item.price_numeric.is_some())
                        .min_by(|a, b| {
                            a.price_numeric.unwrap()
                                .partial_cmp(&b.price_numeric.unwrap())
                                .unwrap_or(std::cmp::Ordering::Equal)
                        });
                    Ok::<Option<GpuListing>, anyhow::Error>(cheapest)
                })().await;
                (model, res)
            }
        });
        let results = futures::future::join_all(cheapest_futures).await;
        for (model, res) in results {
            match res {
                Ok(Some(listing)) => final_listings.push(listing),
                Ok(None) => {
                    if !quiet {
                        println!("No available listing with a valid price found for {:?}", model);
                    }
                },
                Err(e) => {
                    if !quiet {
                        eprintln!("Warning: Failed to process model {:?}: {:?}", model, e);
                    }
                }
            }
        }
    } else {
        let url = format!("{}{}", scraper::BASE_URL, args.gpu);
        let html = web::fetch_html(&url, quiet).await?;
        let mut listings = scraper::parse_listings(&html, quiet)?;
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
        final_listings = listings;
    }

    if !final_listings.is_empty() {
        if !quiet {
            println!(
                "Sorting results by {:?} {}...",
                args.sort_by,
                if args.desc { "descending" } else { "ascending" }
            );
        }
        final_listings.sort_by(|a, b| {
            let ordering = match args.sort_by {
                SortColumn::Name => a.name.cmp(&b.name),
                SortColumn::Status => a.status.cmp(&b.status),
                SortColumn::Price => match (a.price_numeric, b.price_numeric) {
                    (Some(pa), Some(pb)) => pa.partial_cmp(&pb).unwrap_or(std::cmp::Ordering::Equal),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => a.price.cmp(&b.price),
                },
                SortColumn::LastAvailable => a.last_available.cmp(&b.last_available),
                SortColumn::Link => a.link.cmp(&b.link),
            };
            if args.desc { ordering.reverse() } else { ordering }
        });
    }

    if let Some(limit) = args.limit {
        if limit < final_listings.len() {
            if !quiet {
                println!("Limiting results to the top {} listings.", limit);
            }
            final_listings.truncate(limit);
        }
    }

    match args.format {
        OutputFormat::Table => output::print_table(&final_listings, &args.sort_by, args.desc),
        OutputFormat::Json => output::print_json(&final_listings)?,
        OutputFormat::Yaml => output::print_yaml(&final_listings)?,
        OutputFormat::Toml => output::print_toml(&final_listings)?,
    }
    Ok(())
}