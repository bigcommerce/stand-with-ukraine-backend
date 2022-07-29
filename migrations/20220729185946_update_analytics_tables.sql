-- Rename analytics table and add columns
ALTER TABLE
	charity_visited_events RENAME TO charity_events;

ALTER TABLE
	charity_events
ADD
	event_type VARCHAR(25) NOT NULL;
