use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Data {
    Int(i32),
    Float(f64),
    Text(String),
    List(Vec<Data>),
    Map(HashMap<String, Data>),
}

pub const MDL_TO_EUR: f64 = 0.052; // Conversion rate from MDL to EUR
pub const EUR_TO_MDL: f64 = 19.24;

impl Data {
    // Serialize Data to the custom Bracketed-Indented (BI) format
    pub fn to_bi(&self, indent: usize) -> String {
        let indent_str = " ".repeat(indent);

        match self {
            Data::Int(i) => format!("{}{}", indent_str, i),
            Data::Float(f) => format!("{}{}", indent_str, f),
            Data::Text(s) => format!("{}\"{}\"", indent_str, s),
            Data::List(items) => {
                let bi_items: Vec<String> = items.iter().map(|item| item.to_bi(indent + 4)).collect();
                format!("{}[\n{}\n{}]", indent_str, bi_items.join("\n"), indent_str)
            }
            Data::Map(map) => {
                let bi_map: Vec<String> = map
                    .iter()
                    .map(|(k, v)| format!("{}{} [\n{}\n{}]", indent_str, k, v.to_bi(indent + 4), indent_str))
                    .collect();
                bi_map.join("\n")
            }
        }
    }
}