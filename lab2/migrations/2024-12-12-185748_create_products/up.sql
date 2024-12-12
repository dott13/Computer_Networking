-- Your SQL goes here
CREATE TABLE products (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR NOT NULL,
    price DECIMAL(10, 2) NOT NULL,
    description TEXT,
    image BLOB
);