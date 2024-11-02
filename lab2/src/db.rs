use diesel::r2d2::{self, ConnectionManager};
use diesel::SqliteConnection;
use std::env;

pub type DbConnection = r2d2::Pool<ConnectionManager<SqliteConnection>>;

pub fn establish_connection() -> DbConnection {
    // Read the DATABASE_URL environment variable
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    println!("{}",database_url);
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    r2d2::Pool::builder().build(manager).expect("Failed to create pool.")
}