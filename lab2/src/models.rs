use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::products)]
pub struct Product {
    pub id: Option<i32>,
    pub name: String,
    pub price: f64,
    pub description: Option<String>,
    pub image: Option<Vec<u8>>,
}

#[derive(Insertable, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::products)]
pub struct NewProduct {
    pub name: String,
    pub price: f64,
    pub description: Option<String>,
    pub image: Option<Vec<u8>>,
}