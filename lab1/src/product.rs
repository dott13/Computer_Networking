use std::collections::HashMap;
use crate::data::Data;

#[derive(Debug)]
pub struct Product {
    pub name: String,
    pub price: f64,
    pub link: String,
    pub description: String,
}

impl Product {
    // Serialize to custom Brackets Indent Format
    fn to_data(&self) -> Data {
        let mut map = HashMap::new();
        map.insert("name".to_string(), Data::Text(self.name.clone()));
        map.insert("price".to_string(), Data::Float(self.price));
        map.insert("link".to_string(), Data::Text(self.link.clone()));
        Data::Map(map)
    }

    pub fn serialize_products_to_bi(products: &[Product]) -> String {
        let data_list: Vec<Data> = products.iter().map(|p| p.to_data()).collect();
        Data::List(data_list).to_bi(0)
    }

    // Serialize a single product to JSON
    pub fn to_json(&self) -> String {
        format!(
            r#"{{
    "name": "{}",
    "price": {},
    "link": "{}",
    "attribut": "{}"
}}"#,
            // Escape special characters in JSON strings
            self.name.replace('"', "\\\""),
            self.price,
            self.link.replace('"', "\\\""),
            self.description,
        )
    }

    // Serialize a single product to XML
    pub fn to_xml(&self) -> String {
        format!(
            r#"<product>
    <name>{}</name>
    <price>{}</price>
    <link>{}</link>
    <attribute{}</attribute>
</product>"#,
            // Escape special characters in XML
            escape_xml(&self.name),
            self.price,
            escape_xml(&self.link),
            escape_xml(&self.description)
        )
    }
}

// Helper function to escape special XML characters
fn escape_xml(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

// Serialize a vector of products to JSON
pub fn serialize_products_to_json(products: &[Product]) -> String {
    let products_json: Vec<String> = products
        .iter()
        .map(|product| product.to_json())
        .collect();
    
    format!(
        r#"{{
    "timestamp": "{}",
    "products": [
        {}
    ]
}}"#,
        chrono::Utc::now().to_rfc3339(),
        products_json.join(",\n        ")
    )
}

// Serialize a vector of products to XML
pub fn serialize_products_to_xml(products: &[Product]) -> String {
    let products_xml: String = products
        .iter()
        .map(|product| product.to_xml())
        .collect::<Vec<String>>()
        .join("\n    ");
    
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<productList>
    <timestamp>{}</timestamp>
    {}
</productList>"#,
        chrono::Utc::now().to_rfc3339(),
        products_xml
    )
}

// Serialize products into BI format
pub fn serialize_products_to_bi(products: &[Product]) -> String {
    let bi_products: Vec<String> = products.iter().map(|product| {
        let mut product_map = HashMap::new();
        product_map.insert("name".to_string(), Data::Text(product.name.clone()));
        product_map.insert("price".to_string(), Data::Float(product.price));
        product_map.insert("link".to_string(), Data::Text(product.link.clone()));
        product_map.insert("attributes".to_string(), Data::Text(product.description.clone()));

        let product_data = Data::Map(product_map);
        product_data.to_bi(4)
    }).collect();

    format!("Products [\n{}\n]", bi_products.join("\n"))
}