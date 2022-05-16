-- Create the unpublish event table
CREATE TABLE unpublish_events(
	id bigserial PRIMARY KEY,
	store_hash VARCHAR(25) NOT NULL references stores(store_hash),
	unpublished_at timestamptz NOT NULL,
	reason VARCHAR(400) NOT NULL
);