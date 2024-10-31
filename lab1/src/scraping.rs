// scraping.rs

use reqwest;
use select::document::Document;
use select::predicate::{Name, Class};
use std::error::Error;
use crate::data::{Product};
use crate::validation::{validate_product_name, validate_price, convert_price_to_numeric};

pub fn scrape_products(url: &str) -> Result<Vec<Product>, Box<dyn Error>> {
    // Fetch the webpage
    let body = reqwest::blocking::get(url)?.text()?;

    // Parse the HTML document
    let document = Document::from(body.as_str());

    // Collect products into a vector
    let mut products: Vec<Product> = Vec::new();

    // Loop through each product figure
    for node in document.find(Name("figure")) {
        // Attempt to find the product name
        let product_name = node.find(Name("a"))
            .filter(|n| n.attr("class").map_or(false, |c| c.contains("xp-title")))
            .next()
            .map(|n| n.text())
            .unwrap_or_else(|| "Product name not found".to_string());

        // Attempt to find the price
        let price = node.find(Class("xprice"))
            .next()
            .map(|n| n.text())
            .unwrap_or_else(|| "Price not found".to_string());

        // Attempt to find the product link
        let product_link = node.find(Name("a"))
            .filter(|n| n.attr("class").map_or(false, |c| c.contains("xp-title")))
            .next()
            .and_then(|n| n.attr("href"))
            .unwrap_or("Link not found");

        // Validate product name and price
        if validate_product_name(&product_name) && validate_price(&price) {
            // Convert price to a numeric format
            let numeric_price = convert_price_to_numeric(&price)?;

            // Add product to the vector
            products.push(Product {
                name: product_name.clone(),
                price: numeric_price,
                link: product_link.to_string(),
            });
        } else {
            // Print validation errors
            if !validate_product_name(&product_name) {
                println!("Validation failed: Product name is empty.");
            }
            if !validate_price(&price) {
                println!("Validation failed: Price is not a valid number.");
            }
        }
    }

    Ok(products)
}
