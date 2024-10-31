// validation.rs

use std::error::Error;

pub fn validate_product_name(name: &str) -> bool {
    !name.is_empty() // Ensure the name is not empty
}

pub fn validate_price(price: &str) -> bool {
    // Check if the price is a valid number (allowing for optional decimals)
    let cleaned_price = price.replace("lei", "").replace(" ", ""); // Clean the price string
    !cleaned_price.is_empty() && cleaned_price.chars().all(|c| c.is_digit(10) || c == '.')
}

pub fn convert_price_to_numeric(price: &str) -> Result<f64, Box<dyn Error>> {
    let cleaned_price = price.replace("lei", "").replace(" ", "").replace(",", "."); // Clean the price string
    let price_value: f64 = cleaned_price.trim().parse()?;

    // Check if price is in MDL or EUR and convert accordingly
    if price.contains("EUR") {
        Ok(price_value * super::data::EUR_TO_MDL) // Convert EUR to MDL
    } else {
        Ok(price_value) // Assume it's already in MDL
    }
}
