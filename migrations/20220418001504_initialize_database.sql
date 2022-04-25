-- Create the store table
CREATE TABLE stores(
	id uuid NOT NULL PRIMARY KEY,
	store_hash VARCHAR(25) NOT NULL UNIQUE,
	access_token VARCHAR(100) NOT NULL,
	installed_at timestamptz NOT NULL
)