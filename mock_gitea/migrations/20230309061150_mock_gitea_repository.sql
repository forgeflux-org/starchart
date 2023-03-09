CREATE TABLE IF NOT EXISTS mock_gitea_users (
    name VARCHAR(64) NOT NULL UNIQUE,
    ID INTEGER PRIMARY KEY NOT NULL
);

CREATE TABLE IF NOT EXISTS mock_gitea_repositories (
    name VARCHAR(64) NOT NULL UNIQUE,
    user_id INTEGER NOT NULL REFERENCES mock_gitea_users(ID) ON DELETE CASCADE,
    ID INTEGER PRIMARY KEY NOT NULL
);

CREATE TABLE IF NOT EXISTS mock_gitea_tags (
    name VARCHAR(64) NOT NULL UNIQUE,
    ID INTEGER PRIMARY KEY NOT NULL
);


CREATE TABLE IF NOT EXISTS mock_gitea_tag_repo_mapping (
    repository_id INTEGER NOT NULL REFERENCES mock_gitea_repositories(ID) ON DELETE CASCADE,
    topic_id INTEGER NOT NULL REFERENCES mock_gitea_tags(ID) ON DELETE CASCADE,
    PRIMARY KEY (repository_id, topic_id)
);


CREATE VIEW IF NOT EXISTS mock_gitea_view_repos
AS
    SELECT
        mock_gitea_users.name as username,
        mock_gitea_users.ID as user_id,
        mock_gitea_repositories.NAME as name,
        mock_gitea_repositories.ID as repo_id
FROM
    mock_gitea_repositories
INNER JOIN mock_gitea_users ON mock_gitea_repositories.user_id = mock_gitea_users.ID;


CREATE VIEW IF NOT EXISTS mock_gitea_view_repos_tags
AS
    SELECT
        mock_gitea_repositories.name as name,
        mock_gitea_users.name as username,
        mock_gitea_tags.name as tag
FROM
    mock_gitea_tag_repo_mapping
INNER JOIN
    mock_gitea_repositories
ON
    mock_gitea_tag_repo_mapping.repository_id = mock_gitea_repositories.ID
INNER JOIN
    mock_gitea_tags
ON
    mock_gitea_tag_repo_mapping.topic_id = mock_gitea_tags.ID
INNER JOIN
    mock_gitea_users
ON
    mock_gitea_repositories.user_id = mock_gitea_users.ID;

