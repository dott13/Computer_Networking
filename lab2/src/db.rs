use diesel::r2d2::{self, ConnectionManager};
use diesel::SqliteConnection;

pub type DbConnection = r2d2::Pool<ConnectionManager<SqliteConnection>>;

pub fn establish_connection() -> DbConnection {
    let manager = ConnectionManager::<SqliteConnection>::new("DATABASE_URL");
    r2d2::Pool::builder().build(manager).expect("Failed to create pool.")
}
