mod product;
mod scraping;
mod validation;
mod data;

use std::error::Error;
use chrono::{DateTime, Utc};
use product::{Product, serialize_products_to_json, serialize_products_to_xml, serialize_products_to_bi};
use scraping::scrape_products;
use data::MDL_TO_EUR;

fn main() -> Result<(), Box<dyn Error>> {
    // Scrape products from the website
    let products = scrape_products("https://xstore.md/")?;
    
    // Process products: Filter, map, and reduce
    let filtered_products: Vec<Product> = products.into_iter()
        .filter(|p| p.price >= 1000.0 && p.price <= 15000.0)
        .collect();

    let total_price_mdl: f64 = filtered_products.iter()
        .map(|p| p.price)
        .sum();
    let total_price_eur = total_price_mdl * MDL_TO_EUR;
    
    // Get the current UTC timestamp
    let readable_timestamp: DateTime<Utc> = Utc::now();
    
    // Serialize and print the filtered products in both formats
    println!("\nJSON Output:");
    println!("{}", serialize_products_to_json(&filtered_products));
    
    println!("\nXML Output:");
    println!("{}", serialize_products_to_xml(&filtered_products));
    
    println!("\nBracket Indent Custom Format Output:");
    println!("{}", serialize_products_to_bi(&filtered_products));

    // Print the summary information
    println!("\nSummary:");
    println!("Total Price of Filtered Products: {:.2} MDL (~ {:.2} EUR)", 
             total_price_mdl, total_price_eur);
    println!("Timestamp: {}", readable_timestamp.to_rfc3339());
    
    // Display details of each filtered product
    println!("\nDetailed Product List:");
    for product in filtered_products {
        println!("Product: {}, Price: {:.2} MDL (~ {:.2} EUR), Link: {}, Details: {}",
                 product.name,
                 product.price,
                 product.price * MDL_TO_EUR,
                 product.link,
                 product.description);
    }
    
    Ok(())
}