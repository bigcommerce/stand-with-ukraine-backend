-- Create the feedback form table
CREATE TABLE feedback_form(
	id bigserial PRIMARY KEY,
	submitted_at timestamptz NOT NULL,
    name VARCHAR(50) NOT NULL,
	email VARCHAR(100) NOT NULL,
	message VARCHAR(1000) NOT NULL
);