-- sqlite3 migrations/20231220231558_inscriptions.sql

-- A table to store inscriptions
CREATE TABLE IF NOT EXISTS inscriptions
(
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    -- transaction data
    txn_data BLOB NOT NULL
);