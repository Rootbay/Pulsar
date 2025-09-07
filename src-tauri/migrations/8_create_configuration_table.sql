-- A simple key-value table to store configuration data like the password salt.
CREATE TABLE IF NOT EXISTS configuration (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);