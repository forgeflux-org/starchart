CREATE TABLE IF NOT EXISTS starchart_introducer (
	ID INTEGER PRIMARY KEY NOT NULL,
	instance_url TEXT NOT NULL UNIQUE
);


ALTER TABLE starchart_forges ADD COLUMN starchart_instance INTEGER REFERENCES
starchart_introducer(ID) ON DELETE CASCADE DEFAULT NULL;
