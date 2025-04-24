use crate::scraper::GpuListing; // Use GpuListing from scraper module
use crate::cli::SortColumn; // Use SortColumn from cli module (will create next)
use anyhow::{Context, Result};
use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Attribute, Cell, CellAlignment, Color,
    ColumnConstraint, ContentArrangement, Table, Width,
};

fn create_status_cell(status: &str) -> Cell {
    let cell = Cell::new(status).set_alignment(CellAlignment::Center);
    match status.to_lowercase().as_str() {
        "in stock" => cell.add_attribute(Attribute::Bold).fg(Color::Green),
        "preorder" => cell.add_attribute(Attribute::Bold).fg(Color::Yellow),
        "out of stock" | "not tracking" => cell.fg(Color::Red),
        "stock available" => cell.fg(Color::DarkGreen), // Handle Ebay status
        _ => cell,
    }
}

pub fn print_table(listings: &[GpuListing], sort_by: &SortColumn, descending: bool) {
    if listings.is_empty() {
        println!("No listings found to display (after filtering).");
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);

    // Adjust constraints - give Name and Link more space
    table.set_constraints(vec![
        ColumnConstraint::LowerBoundary(Width::Fixed(60)), // Name
        ColumnConstraint::LowerBoundary(Width::Fixed(15)), // Status
        ColumnConstraint::LowerBoundary(Width::Fixed(12)), // Price
        ColumnConstraint::LowerBoundary(Width::Fixed(35)), // Last Available
        ColumnConstraint::LowerBoundary(Width::Fixed(40)), // Link
    ]);

    // Helper closure for header formatting
    let header_cell = |name: &str, col: SortColumn| {
        let arrow = if *sort_by == col { if descending { "▼" } else { "▲" } } else { "" };
        Cell::new(format!("{} {}", name, arrow)).add_attribute(Attribute::Bold)
    };

    table.set_header(vec![
        header_cell("Name", SortColumn::Name),
        header_cell("Status", SortColumn::Status).set_alignment(CellAlignment::Center),
        header_cell("Price", SortColumn::Price).set_alignment(CellAlignment::Right),
        header_cell("Last Available", SortColumn::LastAvailable).set_alignment(CellAlignment::Right),
        header_cell("Link", SortColumn::Link),
    ]);

    for item in listings {
        table.add_row(vec![
            Cell::new(&item.name),
            create_status_cell(&item.status),
            Cell::new(&item.price).set_alignment(CellAlignment::Right),
            Cell::new(&item.last_available).set_alignment(CellAlignment::Right),
            Cell::new(&item.link), // Display raw link - terminals usually handle this
        ]);
    }

    println!("{}", table);
}

pub fn print_json(listings: &[GpuListing]) -> Result<()> {
    let json = serde_json::to_string_pretty(listings)
        .context("Failed to serialize listings to JSON")?;
    println!("{}", json);
    Ok(())
}

pub fn print_yaml(listings: &[GpuListing]) -> Result<()> {
    let yaml = serde_yaml::to_string(listings)
        .context("Failed to serialize listings to YAML")?;
    println!("{}", yaml);
    Ok(())
}

pub fn print_toml(listings: &[GpuListing]) -> Result<()> {
    // TOML requires a top-level table. We'll wrap the list in a table named "listings".
    #[derive(serde::Serialize)]
    struct TomlWrapper<'a> {
        listings: &'a [GpuListing],
    }
    let wrapper = TomlWrapper { listings };
    let toml = toml::to_string_pretty(&wrapper)
        .context("Failed to serialize listings to TOML")?;
    println!("{}", toml);
    Ok(())
}