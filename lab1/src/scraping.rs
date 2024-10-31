use std::io::{Read, Write};
use std::net::TcpStream;
use std::error::Error;
use std::str;
use url::Url;
use select::document::Document;
use select::predicate::{Name, Class};
use native_tls::TlsConnector;
use crate::product::Product;
use crate::validation::{validate_product_name, validate_price, convert_price_to_numeric};

// Trait to abstract over different types of streams
trait StreamIO {
    fn read_response(&mut self) -> Result<String, Box<dyn Error>>;
    fn write_request(&mut self, request: &str) -> Result<(), Box<dyn Error>>;
}

// Implementation for TLS streams
impl StreamIO for native_tls::TlsStream<TcpStream> {
    fn read_response(&mut self) -> Result<String, Box<dyn Error>> {
        let mut buffer = Vec::new();
        self.read_to_end(&mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }

    fn write_request(&mut self, request: &str) -> Result<(), Box<dyn Error>> {
        self.write_all(request.as_bytes())?;
        Ok(())
    }
}

// Implementation for regular TCP streams
impl StreamIO for TcpStream {
    fn read_response(&mut self) -> Result<String, Box<dyn Error>> {
        let mut buffer = Vec::new();
        self.read_to_end(&mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }

    fn write_request(&mut self, request: &str) -> Result<(), Box<dyn Error>> {
        self.write_all(request.as_bytes())?;
        Ok(())
    }
}

pub fn scrape_products(initial_url: &str) -> Result<Vec<Product>, Box<dyn Error>> {
    let mut current_url = initial_url.to_string();
    let mut redirect_count = 0;
    const MAX_REDIRECTS: u8 = 5;

    while redirect_count < MAX_REDIRECTS {
        match fetch_url(&current_url) {
            Ok((status, headers, body)) => {
                if status >= 300 && status < 400 {
                    if let Some(location) = headers.iter()
                        .find(|line| line.to_lowercase().starts_with("location:"))
                        .and_then(|line| line.split(": ").nth(1)) {
                        current_url = location.trim().to_string();
                        redirect_count += 1;
                        println!("Following redirect ({}) to: {}", status, current_url);
                        continue;
                    }
                }
                return parse_products(&body);
            }
            Err(e) => {
                eprintln!("Error fetching URL {}: {}", current_url, e);
                return Err(e);
            }
        }
    }

    Err("Too many redirects".into())
}

fn scrape_product_details(product_link: &str) -> Result<String, Box<dyn Error>> {
    let (status, _, body) = fetch_url(product_link)?;
    
    if status != 200 {
        return Err("Failed to fetch product details".into());
    }

    let document = Document::from(body.as_str());
    
    // Extract the product attributes
    let attributes = document.find(Class("xp-attr"))
        .next()
        .map(|n| n.text())
        .unwrap_or_else(|| "Attributes not found".to_string());

    Ok(attributes)
}



fn fetch_url(url: &str) -> Result<(u32, Vec<String>, String), Box<dyn Error>> {
    let parsed_url = Url::parse(url)?;
    let host = parsed_url.host_str().ok_or("Invalid host")?;
    let port = if parsed_url.scheme() == "https" { 443 } else { 80 };
    let path = if parsed_url.path().is_empty() { "/" } else { parsed_url.path() };

    let request = format!(
        "GET {} HTTP/1.1\r\n\
        Host: {}\r\n\
        User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36\r\n\
        Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8\r\n\
        Connection: close\r\n\r\n",
        path,
        host
    );

    let response = if parsed_url.scheme() == "https" {
        let connector = TlsConnector::new()?;
        let tcp_stream = TcpStream::connect(format!("{}:{}", host, port))?;
        tcp_stream.set_read_timeout(Some(std::time::Duration::from_secs(10)))?;
        let mut tls_stream = connector.connect(host, tcp_stream)?;
        tls_stream.write_request(&request)?;
        tls_stream.read_response()?
    } else {
        let mut stream = TcpStream::connect(format!("{}:{}", host, port))?;
        stream.set_read_timeout(Some(std::time::Duration::from_secs(10)))?;
        stream.write_request(&request)?;
        stream.read_response()?
    };

    // Split headers and body
    let parts: Vec<&str> = response.split("\r\n\r\n").collect();
    if parts.len() < 2 {
        return Err("Invalid response format".into());
    }

    let headers: Vec<String> = parts[0]
        .lines()
        .map(|s| s.to_string())
        .collect();

    // Parse status code
    let status_line = headers.first().ok_or("No status line")?;
    let status_code = status_line
        .split_whitespace()
        .nth(1)
        .ok_or("No status code")?
        .parse::<u32>()?;

    Ok((status_code, headers, parts[1].to_string()))
}

fn parse_products(body: &str) -> Result<Vec<Product>, Box<dyn Error>> {
    let document = Document::from(body);
    let mut products = Vec::new();

    let product_nodes: Vec<_> = document.find(Name("figure")).collect();
    println!("Found {} product nodes", product_nodes.len());

    for node in product_nodes {
        let product_name = node.find(Name("a"))
            .filter(|n| n.attr("class").map_or(false, |c| c.contains("xp-title")))
            .next()
            .map(|n| n.text())
            .unwrap_or_else(|| "Product name not found".to_string());

        let price = node.find(Class("xprice"))
            .next()
            .map(|n| n.text())
            .unwrap_or_else(|| "Price not found".to_string());

        let product_link = node.find(Name("a"))
            .filter(|n| n.attr("class").map_or(false, |c| c.contains("xp-title")))
            .next()
            .and_then(|n| n.attr("href"))
            .unwrap_or("Link not found");

            if validate_product_name(&product_name) && validate_price(&price) {
                if let Ok(numeric_price) = convert_price_to_numeric(&price) {
                    // Fetch description from the product link
                    let attributes = scrape_product_details(product_link)
                        .unwrap_or_else(|_| "Attributes not found".to_string());
            
                    products.push(Product {
                        name: product_name,
                        price: numeric_price,
                        link: product_link.to_string(),
                        description: attributes, // This is now of type Option<String>
                    });
                }
            }
            
    }

    Ok(products)
}
