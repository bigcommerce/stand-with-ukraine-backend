-- Increase the reason column length
ALTER TABLE
	unpublish_events
ALTER COLUMN
	reason TYPE varchar(1000);