use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;

mod models;
mod schema;

fn seed_roles_and_blocks(conn: &mut SqliteConnection) {
    use crate::schema::roles;
    use crate::schema::blocks;

    diesel::insert_into(roles::table)
    .values(&vec![
        (roles::name.eq("admin")),
        (roles::name.eq("resident")),
    ])
    .execute(conn)
    .expect("Failed to seed roles");

    diesel::insert_into(blocks::table)
        .values(&vec![
            (blocks::name.eq("Block 1")),
            (blocks::name.eq("Block 2")),
            (blocks::name.eq("Block 3")),
        ])
        .execute(conn)
        .expect("Failed to seed blocks tabke");
}

fn main() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let mut conn = SqliteConnection::establish(&database_url)
    .expect("Failed to connect to the database");

    // Seed data
    seed_roles_and_blocks(&mut conn);
    println!("Database seeded successfully.")
}