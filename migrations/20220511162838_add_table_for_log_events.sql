-- Create the log tables
CREATE TABLE charity_visited_events(
	id bigserial PRIMARY KEY,
	store_hash VARCHAR(25) NOT NULL references stores(store_hash),
	created_at timestamptz NOT NULL,
	charity VARCHAR(25) NOT NULL
);

CREATE TABLE widget_events(
	id bigserial PRIMARY KEY,
	store_hash VARCHAR(25) NOT NULL references stores(store_hash),
	created_at timestamptz NOT NULL,
	event_type VARCHAR(25) NOT NULL
);