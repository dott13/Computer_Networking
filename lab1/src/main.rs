mod data;
mod scraping;
mod validation;

use std::error::Error;
use chrono::{DateTime, Utc};
use data::{Product, MDL_TO_EUR};
use scraping::scrape_products;

fn main() -> Result<(), Box<dyn Error>> {
    // Scrape products from the website
    let products = scrape_products("https://xstore.md")?;

    // Process products: Filter, map, and reduce
    let filtered_products: Vec<&Product> = products.iter()
        .filter(|p| p.price >= 1000.0 && p.price <= 15000.0) // Example price range
        .collect();

    let total_price_mdl: f64 = filtered_products.iter()
        .map(|p| p.price) // Map prices to MDL
        .sum(); // Reduce to total sum

    let total_price_eur = total_price_mdl * MDL_TO_EUR; // Convert total price to EUR

    // Get the current UTC timestamp
    let readable_timestamp: DateTime<Utc> = Utc::now();

    // Print the results
    println!("Filtered Products: {:?}", filtered_products);
    println!("Total Price of Filtered Products: {:.2} MDL (~ {:.2} EUR)", total_price_mdl, total_price_eur);
    println!("Timestamp: {}", readable_timestamp.to_rfc3339()); // Print readable timestamp

    // Display details of each filtered product
    for product in filtered_products {
        println!("Product: {}, Price: {:.2} MDL (~ {:.2} EUR), Link: {}", 
                 product.name, 
                 product.price, 
                 product.price * MDL_TO_EUR, 
                 product.link);
    }

    Ok(())
}
