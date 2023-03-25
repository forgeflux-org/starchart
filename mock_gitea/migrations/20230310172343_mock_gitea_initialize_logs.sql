CREATE TABLE IF NOT EXISTS mock_gitea_initialize_logs (
    num_tags INTEGER NOT NULL,
    num_users INTEGER NOT NULL,
    num_repos INTEGER NOT NULL,
    initialize_time INTEGER NOT NULL,
    ID INTEGER PRIMARY KEY NOT NULL
);
