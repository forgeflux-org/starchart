/*
 * Copyright (C) 2022  Aravinth Manivannan <realaravinth@batsense.net>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
#[cfg(not(test))]
use log::debug;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;

#[cfg(test)]
use println as debug;

use crate::errors::*;
use crate::settings::Settings;

pub type BoxDB = Box<Database>;

pub async fn get_data(settings: Option<Settings>) -> BoxDB {
    let settings = settings.unwrap_or_else(|| Settings::new().unwrap());

    let pool = settings.database.pool;
    let pool_options = SqlitePoolOptions::new().max_connections(pool);
    let connection_options = ConnectionOptions::Fresh(Fresh {
        pool_options,
        url: settings.database.url,
    });

    let db = connection_options.connect().await.unwrap();
    db.migrate().await.unwrap();
    Box::new(db)
}

#[derive(Clone)]
pub struct Database {
    pub pool: SqlitePool,
}

/// Use an existing database pool
pub struct Conn(pub SqlitePool);

/// Connect to databse
pub enum ConnectionOptions {
    /// fresh connection
    Fresh(Fresh),
    /// existing connection
    Existing(Conn),
}

pub struct Fresh {
    pub pool_options: SqlitePoolOptions,
    pub url: String,
}

impl ConnectionOptions {
    async fn connect(self) -> ServiceResult<Database> {
        use sqlx::sqlite::SqliteConnectOptions;
        use std::str::FromStr;

        let pool = match self {
            Self::Fresh(fresh) => fresh
                .pool_options
                .connect_with(
                    SqliteConnectOptions::from_str(&fresh.url)
                        .unwrap()
                        .create_if_missing(true)
                        .read_only(false),
                )
                .await
                .unwrap(),
            Self::Existing(conn) => conn.0,
        };
        Ok(Database { pool })
    }
}

pub struct AddRepository<'a> {
    pub username: &'a str,
    pub name: &'a str,
    pub tags: &'a [&'a str],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Repository {
    pub username: String,
    pub user_id: i64,
    pub repo_id: i64,
    pub name: String,
}

struct InnerRepository {
    username: Option<String>,
    user_id: Option<i64>,
    repo_id: Option<i64>,
    name: Option<String>,
}

struct InnerTag {
    tag: String,
}

impl InnerRepository {
    fn to_repository(self) -> Repository {
        Repository {
            username: self.username.unwrap(),
            user_id: self.user_id.unwrap(),
            repo_id: self.repo_id.unwrap(),
            name: self.name.unwrap(),
        }
    }
}

impl Database {
    pub async fn migrate(&self) -> ServiceResult<()> {
        sqlx::migrate!("./migrations/")
            .run(&self.pool)
            .await
            .unwrap();

        Ok(())
    }

    pub async fn add_user(&self, username: &str) -> ServiceResult<()> {
        sqlx::query!(
            "INSERT OR IGNORE INTO mock_gitea_users (name) VALUES ($1);",
            username
        )
        .execute(&self.pool)
        .await
        .unwrap();
        Ok(())
    }

    pub async fn add_repository(&self, msg: AddRepository<'_>) -> ServiceResult<()> {
        self.add_user(msg.username).await?;

        sqlx::query!(
            "INSERT OR IGNORE INTO mock_gitea_repositories
            (name, user_id) VALUES (
                $1,
                (SELECT ID FROM mock_gitea_users WHERE name = $2)
            );",
            msg.name,
            msg.username
        )
        .execute(&self.pool)
        .await
        .unwrap();

        for tag in msg.tags.iter() {
            self.add_tag(tag).await?;
            sqlx::query!(
                "INSERT OR IGNORE INTO
                    mock_gitea_tag_repo_mapping
                (repository_id, topic_id) VALUES (
                    (SELECT ID FROM mock_gitea_repositories WHERE name = $1),
                    (SELECT ID FROM mock_gitea_tags WHERE name = $2)
                );",
                msg.name,
                tag,
            )
            .execute(&self.pool)
            .await
            .unwrap();
        }

        Ok(())
    }

    pub async fn add_tag(&self, tag: &str) -> ServiceResult<()> {
        debug!("Adding tag: {tag}");
        sqlx::query!(
            "INSERT OR IGNORE INTO mock_gitea_tags (name) VALUES ($1);",
            tag
        )
        .execute(&self.pool)
        .await
        .unwrap();
        Ok(())
    }

    pub async fn get_repositories(
        &self,
        offset: u32,
        limit: u32,
    ) -> ServiceResult<Vec<Repository>> {
        let mut repos = sqlx::query_as!(
            InnerRepository,
            "SELECT
                username,
                user_id,
                name,
                repo_id
            FROM
                mock_gitea_view_repos
            ORDER BY
                mock_gitea_view_repos.user_id
            LIMIT $1 OFFSET $2;
        ",
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .unwrap();

        let mut res = Vec::with_capacity(repos.len());

        for repo in repos.drain(0..) {
            res.push(repo.to_repository());
        }

        Ok(res)
    }

    pub async fn get_tags(&self, owner: &str, repo_name: &str) -> ServiceResult<Vec<String>> {
        let mut db_tags = sqlx::query_as!(
            InnerTag,
            "SELECT
                    tag
                FROM
                    mock_gitea_view_repos_tags
                WHERE
                    name = $1
                AND
                    username = $2
            ",
            repo_name,
            owner
        )
        .fetch_all(&self.pool)
        .await
        .unwrap();
        let tags = db_tags.drain(0..).map(|t| t.tag).collect();
        Ok(tags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::*;

    #[actix_rt::test]
    async fn db_works() {
        const USERNAME: &str = "db_works";
        const REPO_NAMES: [&str; 4] = [
            "db_works_repo_1",
            "db_works_repo_2",
            "db_works_repo_3",
            "db_works_repo_4",
        ];
        const TAGS: [&str; 4] = [
            "db_works_tag_1",
            "db_works_tag_2",
            "db_works_tag_3",
            "db_works_tag_4",
        ];

        let (db, _ctx) = sqlx_sqlite::get_ctx().await;

        for name in REPO_NAMES.iter() {
            let msg = AddRepository {
                name,
                username: USERNAME,
                tags: &TAGS,
            };

            db.add_repository(msg).await.unwrap();
        }

        let repos = db.get_repositories(0, 100).await.unwrap();
        assert!(repos.len() > REPO_NAMES.len());

        for repo in repos.iter() {
            let tags = db.get_tags(USERNAME, &repo.name).await.unwrap();
            assert_eq!(tags.len(), TAGS.len());
            assert_eq!(repo.username, USERNAME);
        }
    }
}
