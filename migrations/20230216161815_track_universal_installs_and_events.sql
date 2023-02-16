-- Add universal analytics tables
CREATE TABLE universal_installer_events(
	id bigserial PRIMARY KEY,
	submitted_at timestamptz NOT NULL,
    event_type VARCHAR(50) NOT NULL,
	metadata VARCHAR(100)
);

ALTER TABLE charity_events
ALTER COLUMN store_hash DROP NOT NULL;

ALTER TABLE widget_events
ALTER COLUMN store_hash DROP NOT NULL;