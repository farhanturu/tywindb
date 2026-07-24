-- E-commerce Database Example
-- This script creates a simple e-commerce database.

-- Create tables
CREATE TABLE products (
    id INTEGER,
    name TEXT,
    description TEXT,
    price FLOAT,
    stock INTEGER,
    category TEXT
);

CREATE TABLE customers (
    id INTEGER,
    name TEXT,
    email TEXT,
    phone TEXT
);

CREATE TABLE orders (
    id INTEGER,
    customer_id INTEGER,
    total FLOAT,
    status TEXT,
    created_at TEXT
);

CREATE TABLE order_items (
    id INTEGER,
    order_id INTEGER,
    product_id INTEGER,
    quantity INTEGER,
    price FLOAT
);

-- Insert products
INSERT INTO products (id, name, description, price, stock, category)
VALUES (1, 'Laptop', 'High-performance laptop', 999.99, 10, 'Electronics');

INSERT INTO products (id, name, description, price, stock, category)
VALUES (2, 'Mouse', 'Wireless mouse', 29.99, 100, 'Electronics');

INSERT INTO products (id, name, description, price, stock, category)
VALUES (3, 'Keyboard', 'Mechanical keyboard', 79.99, 50, 'Electronics');

INSERT INTO products (id, name, description, price, stock, category)
VALUES (4, 'Book', 'Programming book', 39.99, 200, 'Books');

-- Insert customers
INSERT INTO customers (id, name, email, phone)
VALUES (1, 'Alice Smith', 'alice@example.com', '555-0101');

INSERT INTO customers (id, name, email, phone)
VALUES (2, 'Bob Johnson', 'bob@example.com', '555-0102');

-- Insert orders
INSERT INTO orders (id, customer_id, total, status, created_at)
VALUES (1, 1, 1029.98, 'completed', '2026-01-20');

INSERT INTO orders (id, customer_id, total, status, created_at)
VALUES (2, 2, 49.98, 'pending', '2026-01-21');

-- Insert order items
INSERT INTO order_items (id, order_id, product_id, quantity, price)
VALUES (1, 1, 1, 1, 999.99);

INSERT INTO order_items (id, order_id, product_id, quantity, price)
VALUES (2, 1, 2, 1, 29.99);

INSERT INTO order_items (id, order_id, product_id, quantity, price)
VALUES (3, 2, 4, 1, 39.99);

INSERT INTO order_items (id, order_id, product_id, quantity, price)
VALUES (4, 2, 2, 1, 29.99);

-- Query examples

-- Get all products with stock > 0
SELECT id, name, price, stock FROM products WHERE stock > 0;

-- Get products by category
SELECT * FROM products WHERE category = 'Electronics';

-- Get order details
SELECT 
    o.id as order_id,
    c.name as customer,
    o.total,
    o.status
FROM orders o
JOIN customers c ON o.customer_id = c.id;

-- Get order items with product names
SELECT 
    oi.order_id,
    p.name as product,
    oi.quantity,
    oi.price
FROM order_items oi
JOIN products p ON oi.product_id = p.id;

-- Get customer order history
SELECT 
    c.name as customer,
    COUNT(o.id) as order_count,
    SUM(o.total) as total_spent
FROM customers c
LEFT JOIN orders o ON c.id = o.customer_id
GROUP BY c.id;

-- Update product stock
UPDATE products SET stock = stock - 1 WHERE id = 1;

-- Update order status
UPDATE orders SET status = 'shipped' WHERE id = 1;
