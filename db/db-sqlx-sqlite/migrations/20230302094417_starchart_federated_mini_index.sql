CREATE VIRTUAL TABLE IF NOT EXISTS starchart_federated_mini_index USING fts4 (
    starchart_instance INTEGER REFERENCES starchart_introducer(ID) ON DELETE CASCADE,
    mini_index TEXT NOT NULL,
    ID INTEGER PRIMARY KEY NOT NULL
);
