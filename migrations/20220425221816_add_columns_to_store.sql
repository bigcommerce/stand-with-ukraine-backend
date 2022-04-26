-- Add columns for uninstalled status, published status and widget_configuration
ALTER TABLE
	stores
ADD
	uninstalled boolean NOT NULL DEFAULT false,
ADD
	published boolean NOT NULL DEFAULT false,
ADD
	widget_configuration jsonb NOT NULL DEFAULT '{}' :: jsonb;