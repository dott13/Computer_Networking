// data.rs

#[derive(Debug)]
pub struct Product {
    pub name: String,
    pub price: f64,
    pub link: String,
}

pub const MDL_TO_EUR: f64 = 0.052; // Conversion rate from MDL to EUR
pub const EUR_TO_MDL: f64 = 19.24; // Conversion rate from EUR to MDL
