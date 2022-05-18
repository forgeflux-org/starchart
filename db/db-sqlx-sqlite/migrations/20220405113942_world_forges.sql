CREATE TABLE IF NOT EXISTS starchart_forge_type (
    name VARCHAR(30) NOT NULL UNIQUE,
    ID INTEGER PRIMARY KEY NOT NULL
);

INSERT OR IGNORE INTO starchart_forge_type (name) VALUES('gitea');

CREATE TABLE IF NOT EXISTS starchart_forges (
	forge_type INTEGER NOT NULL REFERENCES starchart_forge_type(ID) ON DELETE CASCADE,
	hostname TEXT NOT NULL UNIQUE,
	verified_on INTEGER  NOT NULL,
	last_crawl_on INTEGER  DEFAULT NULL,
	ID INTEGER PRIMARY KEY NOT NULL
);

CREATE TABLE IF NOT EXISTS starchart_dns_challenges (
	hostname TEXT NOT NULL UNIQUE,
	challenge TEXT NOT NULL UNIQUE,
	created INTEGER  NOT NULL,
	ID INTEGER PRIMARY KEY NOT NULL
);

CREATE TABLE IF NOT EXISTS starchart_users (
	hostname_id INTEGER NOT NULL REFERENCES starchart_forges(ID) ON DELETE CASCADE,
	username TEXT NOT NULL,
	html_url TEXT NOT NULL UNIQUE,
	profile_photo_html_url TEXT DEFAULT NULL,
	added_on INTEGER NOT NULL,
	last_crawl_on INTEGER NOT NULL,
	ID INTEGER PRIMARY KEY NOT NULL
);

CREATE TABLE IF NOT EXISTS starchart_project_topics (
	name VARCHAR(50) NOT NULL UNIQUE,
	ID INTEGER PRIMARY KEY NOT NULL
);

CREATE TABLE IF NOT EXISTS starchart_repositories (
	ID INTEGER PRIMARY KEY NOT NULL,
	hostname_id INTEGER NOT NULL REFERENCES starchart_forges(ID) ON DELETE CASCADE,
	owner_id INTEGER NOT NULL REFERENCES starchart_users(ID) ON DELETE CASCADE,
	name TEXT NOT NULL,
	description TEXT DEFAULT NULL,
	website TEXT DEFAULT NULL,
	html_url TEXT NOT NULL UNIQUE,
	created INTEGER NOT NULL,
	last_crawl INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS starchart_repository_topic_mapping (
	repository_id INTEGER NOT NULL REFERENCES starchart_repositories(ID) ON DELETE CASCADE,
	topic_id INTEGER NOT NULL REFERENCES starchart_project_topics(ID) ON DELETE CASCADE
);
