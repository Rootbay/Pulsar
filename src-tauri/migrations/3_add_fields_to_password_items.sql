ALTER TABLE password_items ADD COLUMN username TEXT;
ALTER TABLE password_items ADD COLUMN url TEXT;
ALTER TABLE password_items ADD COLUMN notes TEXT;
ALTER TABLE password_items ADD COLUMN password TEXT NOT NULL DEFAULT '';