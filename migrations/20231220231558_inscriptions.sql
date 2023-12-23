-- sqlite3 migrations/20231220231558_inscriptions.sql

-- A table to store inscriptions
CREATE TABLE IF NOT EXISTS inscriptions
(
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    -- sender address of the transaction
    sender              BLOB NOT NULL,
    -- chain id of the transaction
    chain_id            INTEGER NOT NULL,
    -- hash of the transaction
    tx_hash             BLOB NOT NULL,
    -- inscription call data
    calldata            BLOB NOT NULL
);