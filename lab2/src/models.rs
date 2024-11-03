use diesel::{Queryable, Insertable, Selectable};
use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;
use crate::schema::{blocks, roles, users, messages};

#[derive(Queryable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = blocks)]
pub struct Block {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = roles)]
pub struct Role {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Insertable,Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub role_id: Option<i32>,  
    pub apartment: Option<String>,  // Use Option for nullable field
    pub block_id: Option<i32>,  // Use Option for nullable foreign key
    pub password: String,
    pub photo: Option<Vec<u8>>,  // Use Option for nullable BLOB

}

#[derive(Queryable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = messages)]
pub struct Message {
    pub id: i32,
    pub user_id: i32,
    pub block_id: i32,
    pub content: String,
    pub timestamp: Option<NaiveDateTime>,  // Use Option for nullable field
}
