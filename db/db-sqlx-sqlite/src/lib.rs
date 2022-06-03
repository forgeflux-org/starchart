/*
 * ForgeFlux StarChart - A federated software forge spider
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
use std::str::FromStr;

use db_core::dev::*;

use sqlx::sqlite::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::types::time::OffsetDateTime;

pub mod errors;
#[cfg(test)]
pub mod tests;

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

pub mod dev {
    pub use super::errors::*;
    pub use super::Database;
    pub use db_core::dev::*;
    pub use prelude::*;
    pub use sqlx::Error;
}

pub mod prelude {
    pub use super::*;
    pub use db_core::prelude::*;
}

#[async_trait]
impl Connect for ConnectionOptions {
    type Pool = Database;
    async fn connect(self) -> DBResult<Self::Pool> {
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
                .map_err(|e| DBError::DBError(Box::new(e)))?,
            Self::Existing(conn) => conn.0,
        };
        Ok(Database { pool })
    }
}

use dev::*;

#[async_trait]
impl Migrate for Database {
    async fn migrate(&self) -> DBResult<()> {
        sqlx::migrate!("./migrations/")
            .run(&self.pool)
            .await
            .map_err(|e| DBError::DBError(Box::new(e)))?;
        Ok(())
    }
}

#[async_trait]
impl SCDatabase for Database {
    /// ping DB
    async fn ping(&self) -> bool {
        use sqlx::Connection;

        if let Ok(mut con) = self.pool.acquire().await {
            con.ping().await.is_ok()
        } else {
            false
        }
    }

    /// delete forge isntance
    async fn delete_forge_instance(&self, hostname: &str) -> DBResult<()> {
        sqlx::query!(
            "DELETE FROM starchart_forges WHERE hostname = ($1)",
            hostname,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DBError::DBError(Box::new(e)))?;
        Ok(())
    }

    /// create forge isntance DB
    async fn create_forge_isntance(&self, f: &CreateForge) -> DBResult<()> {
        let now = now_unix_time_stamp();
        let forge_type = f.forge_type.to_str();
        sqlx::query!(
            "INSERT INTO
            starchart_forges (hostname, verified_on, forge_type ) 
        VALUES ($1, $2, (SELECT ID FROM starchart_forge_type WHERE name = $3))",
            f.hostname,
            now,
            forge_type,
        )
        .execute(&self.pool)
        .await
        .map_err(map_register_err)?;

        Ok(())
    }

    /// get forge isntance data
    async fn get_forge(&self, hostname: &str) -> DBResult<Forge> {
        struct InnerForge {
            hostname: String,
            last_crawl_on: Option<i64>,
            name: String,
        }
        let f = sqlx::query_as!(
            InnerForge,
            "SELECT 
		hostname,
		last_crawl_on,
		starchart_forge_type.name
        FROM
            starchart_forges
        INNER JOIN
            starchart_forge_type
        ON
            starchart_forges.forge_type = starchart_forge_type.id
        WHERE
            hostname = $1;
            ",
            hostname,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DBError::DBError(Box::new(e)))?;

        let f = Forge {
            hostname: f.hostname,
            last_crawl_on: f.last_crawl_on,
            forge_type: ForgeImplementation::from_str(&f.name).unwrap(),
        };

        Ok(f)
    }

    /// check if a forge instance exists
    async fn forge_exists(&self, hostname: &str) -> DBResult<bool> {
        match sqlx::query!(
            "SELECT ID FROM starchart_forges WHERE hostname = $1",
            hostname
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(_) => Ok(true),
            Err(Error::RowNotFound) => Ok(false),
            Err(e) => Err(DBError::DBError(Box::new(e).into())),
        }
    }

    async fn forge_type_exists(&self, forge_type: &ForgeImplementation) -> DBResult<bool> {
        let forge_type = forge_type.to_str();
        match sqlx::query!(
            "SELECT ID FROM starchart_forge_type WHERE name = $1",
            forge_type
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(_) => Ok(true),
            Err(Error::RowNotFound) => Ok(false),
            Err(e) => Err(DBError::DBError(Box::new(e))),
        }
    }

    /// add new user to database
    async fn add_user(&self, u: &AddUser) -> DBResult<()> {
        let now = now_unix_time_stamp();
        sqlx::query!(
            "INSERT INTO 
                    starchart_users (
                        hostname_id, username, html_url,
                        profile_photo_html_url, added_on, last_crawl_on
                    ) 
            VALUES (
                    (SELECT ID FROM starchart_forges WHERE hostname = $1), $2, $3, $4, $5, $6)",
            u.hostname,
            u.username,
            u.html_link,
            u.profile_photo,
            now,
            now
        )
        .execute(&self.pool)
        .await
        .map_err(map_register_err)?;

        Ok(())
    }

    /// check if an user exists. When hostname of a forge instace is provided, username search is
    /// done only on that forge
    async fn user_exists(&self, username: &str, hostname: Option<&str>) -> DBResult<bool> {
        match hostname {
            Some(hostname) => match sqlx::query!(
                "SELECT ID FROM starchart_users WHERE username = $1 AND 
                hostname_id = (SELECT ID FROM starchart_forges WHERE hostname = $2)",
                username,
                hostname,
            )
            .fetch_one(&self.pool)
            .await
            {
                Ok(_) => Ok(true),
                Err(Error::RowNotFound) => Ok(false),
                Err(e) => Err(DBError::DBError(Box::new(e).into())),
            },
            None => match sqlx::query!(
                "SELECT ID FROM starchart_users WHERE username = $1",
                username
            )
            .fetch_one(&self.pool)
            .await
            {
                Ok(_) => Ok(true),
                Err(Error::RowNotFound) => Ok(false),
                Err(e) => Err(DBError::DBError(Box::new(e).into())),
            },
        }
    }

    /// check if a repo exists.
    async fn repository_exists(&self, name: &str, owner: &str, hostname: &str) -> DBResult<bool> {
        match sqlx::query!(
            "SELECT ID FROM starchart_repositories
                    WHERE 
                        name = $1
                    AND
                        owner_id = ( SELECT ID FROM starchart_users WHERE username = $2)
                    AND
                        hostname_id = (SELECT ID FROM starchart_forges WHERE hostname = $3)",
            name,
            owner,
            hostname,
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(_) => Ok(true),
            Err(Error::RowNotFound) => Ok(false),
            Err(e) => Err(DBError::DBError(Box::new(e).into())),
        }
    }

    /// add new repository to database.
    async fn create_repository(&self, r: &AddRepository) -> DBResult<()> {
        //        unimplemented!()
        let now = now_unix_time_stamp();
        sqlx::query!(
            "INSERT INTO 
                starchart_repositories (
                    hostname_id, owner_id, name, description, html_url, website, created, last_crawl
                )
                VALUES (
                    (SELECT ID FROM starchart_forges WHERE hostname = $1),
                    (SELECT ID FROM starchart_users WHERE username = $2),
                    $3, $4, $5, $6, $7, $8
                );",
            r.hostname,
            r.owner,
            r.name,
            r.description,
            r.html_link,
            r.website,
            now,
            now
        )
        .execute(&self.pool)
        .await
        .map_err(map_register_err)?;

        if let Some(topics) = &r.tags {
            for topic in topics.iter() {
                sqlx::query!(
                    "INSERT OR IGNORE INTO starchart_project_topics ( name ) VALUES ( $1 );",
                    topic,
                )
                .execute(&self.pool)
                .await
                .map_err(map_register_err)?;

                sqlx::query!(
                    "
                        INSERT INTO starchart_repository_topic_mapping ( topic_id, repository_id )
                        VALUES (
                            (SELECT ID FROM starchart_project_topics WHERE name = $1),
                            (SELECT ID FROM starchart_repositories WHERE html_url = $2)
                        );",
                    topic,
                    r.html_link,
                )
                .execute(&self.pool)
                .await
                .map_err(map_register_err)?;
            }
        }

        Ok(())
    }

    /// delete user
    async fn delete_user(&self, username: &str, hostname: &str) -> DBResult<()> {
        sqlx::query!(
            " DELETE FROM starchart_users WHERE username = $1 AND 
                hostname_id = (SELECT ID FROM starchart_forges WHERE hostname = $2)",
            username,
            hostname
        )
        .execute(&self.pool)
        .await
        .map_err(map_register_err)?;
        Ok(())
    }

    /// delete repository
    async fn delete_repository(&self, owner: &str, name: &str, hostname: &str) -> DBResult<()> {
        sqlx::query!(
            " DELETE FROM starchart_repositories
                    WHERE 
                        name = $1
                    AND
                        owner_id = ( SELECT ID FROM starchart_users WHERE username = $2)
                    AND
                        hostname_id = (SELECT ID FROM starchart_forges WHERE hostname = $3)",
            name,
            owner,
            hostname
        )
        .execute(&self.pool)
        .await
        .map_err(map_register_err)?;
        Ok(())
    }

    async fn dns_challenge_exists(&self, key: &str) -> DBResult<bool> {
        match sqlx::query!(
            "SELECT ID FROM starchart_dns_challenges WHERE key = $1",
            key
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(_) => Ok(true),
            Err(Error::RowNotFound) => Ok(false),
            Err(e) => Err(DBError::DBError(Box::new(e).into())),
        }
    }

    async fn get_dns_challenge(&self, key: &str) -> DBResult<Challenge> {
        let res = sqlx::query_as!(
            Challenge,
            "SELECT key, value, hostname FROM starchart_dns_challenges WHERE key = $1",
            key
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DBError::DBError(Box::new(e)))?;
        Ok(res)
    }

    async fn delete_dns_challenge(&self, key: &str) -> DBResult<()> {
        sqlx::query!("DELETE FROM starchart_dns_challenges WHERE key = $1", key)
            .execute(&self.pool)
            .await
            .map_err(map_register_err)?;
        Ok(())
    }

    /// create DNS challenge
    async fn create_dns_challenge(&self, challenge: &Challenge) -> DBResult<()> {
        let now = now_unix_time_stamp();
        sqlx::query!(
            "INSERT INTO
            starchart_dns_challenges (hostname, value, key, created ) 
        VALUES ($1, $2, $3, $4);",
            challenge.hostname,
            challenge.value,
            challenge.key,
            now,
        )
        .execute(&self.pool)
        .await
        .map_err(map_register_err)?;

        Ok(())
    }

    /// Get all repositories
    async fn get_all_repositories(&self, page: u32, limit: u32) -> DBResult<Vec<Repository>> {
        struct InnerRepository {
            /// html link to the repository
            pub html_url: String,
            /// hostname of the forge instance: with scheme but remove trailing slash
            /// hostname can be derived  from html_link also, but used to link to user's forge instance
            pub hostname: String,
            /// repository name
            pub name: String,
            /// repository owner
            pub username: String,
            /// repository description, if any
            pub description: Option<String>,
            /// repository website, if any
            pub website: Option<String>,
            pub ID: i64,
        }

        let mut db_res = sqlx::query_as!(
            InnerRepository,
            "            SELECT 
		starchart_forges.hostname,
		starchart_users.username,
		starchart_repositories.name,
		starchart_repositories.description,
		starchart_repositories.html_url,
        starchart_repositories.ID,
		starchart_repositories.website
FROM
	starchart_repositories
INNER JOIN
	starchart_forges
ON
	starchart_repositories.hostname_id = starchart_forges.id
INNER JOIN
	starchart_users
ON
	starchart_repositories.owner_id =  starchart_users.id
ORDER BY
	starchart_repositories.ID
LIMIT $1 OFFSET $2
;",
            limit,
            page,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_register_err)?;

        let mut res = Vec::with_capacity(db_res.len());
        struct Topics {
            name: String,
        }

        for repo in db_res.drain(0..) {
            let mut db_topics = sqlx::query_as!(
                Topics,
                "SELECT name FROM starchart_project_topics WHERE ID = (
                SELECT topic_id FROM starchart_repository_topic_mapping WHERE repository_id = $1
            )",
                repo.ID
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_register_err)?;

            let topics = if db_topics.is_empty() {
                None
            } else {
                let mut topics = Vec::with_capacity(db_topics.len());
                for t in db_topics.drain(0..) {
                    topics.push(t.name);
                }

                Some(topics)
            };
            res.push(Repository {
                html_url: repo.html_url,
                hostname: repo.hostname,
                name: repo.name,
                username: repo.username,
                description: repo.description,
                website: repo.website,
                tags: topics,
            });
        }

        Ok(res)
    }
}

fn now_unix_time_stamp() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp()
}
