-- Create the store table
CREATE TABLE stores(
	id uuid NOT NULL,
	PRIMARY KEY (id),
	store_hash TEXT NOT NULL UNIQUE,
	installed_at timestamptz NOT NULL
)