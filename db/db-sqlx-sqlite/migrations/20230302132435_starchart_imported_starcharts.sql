CREATE TABLE IF NOT EXISTS starchart_imported_starcharts (
    starchart_instance INTEGER REFERENCES starchart_introducer(ID) ON DELETE CASCADE,
	ID INTEGER PRIMARY KEY NOT NULL
);
