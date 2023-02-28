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
use url::Url;

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

        self.init_project_topics_fts().await?;
        self.init_username_fts().await?;
        self.init_repository_fts().await?;
        Ok(())
    }
}

struct FTSRepository {
    html_url: String,
}

impl Database {
    async fn get_fts_repository(&self, query: &str) -> DBResult<Vec<FTSRepository>> {
        let fts_repos = sqlx::query_as_unchecked!(
            FTSRepository,
            "SELECT html_url FROM fts_repositories WHERE html_url MATCH $1;",
            query
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DBError::DBError(Box::new(e)))?;
        Ok(fts_repos)
    }

    async fn new_fts_repositories(
        &self,
        name: &str,
        description: Option<&str>,
        website: Option<&str>,
        html_url: &str,
    ) -> DBResult<()> {
        if !self.get_fts_repository(html_url).await?.is_empty() {
            return Ok(());
        }
        sqlx::query!(
            "INSERT OR IGNORE INTO fts_repositories ( name, description, website, html_url ) 
            VALUES ( $1, $2, $3, $4 );",
            name,
            description,
            website,
            html_url
        )
        .execute(&self.pool)
        .await
        .map_err(map_register_err)?;
        Ok(())
    }

    async fn init_repository_fts(&self) -> DBResult<()> {
        let limit = 50;
        let mut page = 0;
        loop {
            let offset = page * limit;
            let mut repositories = self.get_all_repositories(offset, limit).await?;
            if repositories.is_empty() {
                break;
            }

            for repo in repositories.drain(0..) {
                self.new_fts_repositories(
                    &repo.name,
                    repo.description.as_ref().map(|d| d.as_str()),
                    repo.website.as_ref().map(|s| s.as_str()),
                    &repo.html_url,
                )
                .await?;
            }
            page += 1;
        }

        Ok(())
    }

    async fn new_fts_user(&self, username: &str) -> DBResult<()> {
        sqlx::query!(
            "INSERT OR IGNORE INTO fts_users ( username ) VALUES ( $1 );",
            username,
        )
        .execute(&self.pool)
        .await
        .map_err(map_register_err)?;
        Ok(())
    }

    async fn init_username_fts(&self) -> DBResult<()> {
        struct User {
            username: String,
        }
        let limit = 50;
        let mut page = 0;
        loop {
            let offset = page * limit;

            let mut users = sqlx::query_as!(
                User,
                "SELECT username FROM starchart_users ORDER BY ID LIMIT $1 OFFSET $2",
                limit,
                offset,
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DBError::DBError(Box::new(e)))?;
            if users.is_empty() {
                break;
            }

            for user in users.drain(0..) {
                self.new_fts_user(&user.username).await?;
            }
            page += 1;
        }

        Ok(())
    }

    async fn new_fts_topic(&self, name: &str) -> DBResult<()> {
        sqlx::query!(
            "INSERT OR IGNORE INTO fts_project_topics ( name ) VALUES ( $1 );",
            name,
        )
        .execute(&self.pool)
        .await
        .map_err(map_register_err)?;
        Ok(())
    }

    async fn init_project_topics_fts(&self) -> DBResult<()> {
        struct Topic {
            name: String,
        }
        let limit = 50;
        let mut page = 0;
        loop {
            let offset = page * limit;
            let mut topics = sqlx::query_as!(
                Topic,
                "SELECT name FROM starchart_project_topics ORDER BY ID LIMIT $1 OFFSET $2;",
                limit,
                offset
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DBError::DBError(Box::new(e)))?;
            if topics.is_empty() {
                break;
            }

            for topic in topics.drain(0..) {
                self.new_fts_topic(&topic.name).await?;
            }
            page += 1;
        }

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

    /// delete forge instance
    async fn delete_forge_instance(&self, url: &Url) -> DBResult<()> {
        let url = db_core::clean_url(url);
        sqlx::query!("DELETE FROM starchart_forges WHERE hostname = ($1)", url,)
            .execute(&self.pool)
            .await
            .map_err(|e| DBError::DBError(Box::new(e)))?;
        Ok(())
    }

    /// create forge instance DB
    async fn create_forge_instance(&self, f: &CreateForge) -> DBResult<()> {
        let now = now_unix_time_stamp();
        let url = db_core::clean_url(&f.url);
        let forge_type = f.forge_type.to_str();
        if let Some(instance_url) = f.starchart_url {
            sqlx::query!(
                "INSERT INTO starchart_forges
                    (hostname, verified_on, forge_type, starchart_instance)
                VALUES (
                        $1,
                        $2,
                        (SELECT ID FROM starchart_forge_type WHERE name = $3),
                        (SELECT ID FROM starchart_introducer WHERE instance_url = $4)
                    )",
                url,
                now,
                forge_type,
                instance_url
            )
            .execute(&self.pool)
            .await
            .map_err(map_register_err)?;
        } else {
            sqlx::query!(
                "INSERT INTO starchart_forges
                    (hostname, verified_on, forge_type, starchart_instance)
                VALUES
                    (
                        $1, $2,
                     (SELECT ID FROM starchart_forge_type WHERE name = $3),
                     $4)",
                url,
                now,
                forge_type,
                f.starchart_url
            )
            .execute(&self.pool)
            .await
            .map_err(map_register_err)?;
        }

        Ok(())
    }

    /// get forge instance data
    async fn get_forge(&self, url: &Url) -> DBResult<Forge> {
        let url = db_core::clean_url(url);
        let f = sqlx::query_as!(
            InnerForge,
            "SELECT 
                hostname,
                last_crawl_on,
                starchart_introducer.instance_url,
                starchart_forge_type.name
            FROM
                starchart_forges
            INNER JOIN
                starchart_forge_type
            ON
                starchart_forges.forge_type = starchart_forge_type.id
            LEFT JOIN
                starchart_introducer
            ON
                starchart_introducer.ID = starchart_forges.starchart_instance
            WHERE
                hostname = $1;
            ",
            url,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DBError::DBError(Box::new(e)))?;

        Ok(f.into())
    }

    /// Get all forges
    async fn get_all_forges(
        &self,
        with_imports: bool,
        offset: u32,
        limit: u32,
    ) -> DBResult<Vec<Forge>> {
        let mut inter_forges = if with_imports {
            sqlx::query_as!(
                InnerForge,
                "SELECT
                hostname,
                last_crawl_on,
                starchart_forge_type.name,
                starchart_introducer.instance_url
            FROM
                starchart_forges
            INNER JOIN
                starchart_forge_type
            ON
                starchart_forges.forge_type = starchart_forge_type.id
            LEFT JOIN
                starchart_introducer
            ON
                starchart_introducer.ID = starchart_forges.starchart_instance
            ORDER BY
                starchart_forges.ID
            LIMIT $1 OFFSET $2;
        ",
                limit,
                offset
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DBError::DBError(Box::new(e)))?
        } else {
            sqlx::query_as!(
                InnerForge,
                "SELECT
                hostname,
                last_crawl_on,
                starchart_introducer.instance_url,
                starchart_forge_type.name
            FROM
                starchart_forges
            INNER JOIN
                starchart_forge_type
            ON
                starchart_forges.forge_type = starchart_forge_type.id
            LEFT JOIN
                starchart_introducer
            ON
                starchart_introducer.ID = starchart_forges.starchart_instance
            WHERE 
                starchart_forges.imported = false
            ORDER BY
                starchart_forges.ID
            LIMIT $1 OFFSET $2;
        ",
                limit,
                offset
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DBError::DBError(Box::new(e)))?
        };

        let mut forges: Vec<Forge> = Vec::with_capacity(inter_forges.len());
        inter_forges.drain(0..).for_each(|f| forges.push(f.into()));

        Ok(forges)
    }

    /// check if a forge instance exists
    async fn forge_exists(&self, url: &Url) -> DBResult<bool> {
        let url = db_core::clean_url(url);
        match sqlx::query!("SELECT ID FROM starchart_forges WHERE hostname = $1", url)
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
        let url = db_core::clean_url(&u.url);
        sqlx::query!(
            "INSERT INTO 
                    starchart_users (
                        hostname_id, username, html_url,
                        profile_photo_html_url, added_on, last_crawl_on, imported
                    ) 
            VALUES (
                    (SELECT ID FROM starchart_forges WHERE hostname = $1), $2, $3, $4, $5, $6, $7)",
            url,
            u.username,
            u.html_link,
            u.profile_photo,
            now,
            now,
            u.import
        )
        .execute(&self.pool)
        .await
        .map_err(map_register_err)?;
        self.new_fts_user(u.username).await?;

        Ok(())
    }

    /// get user data
    async fn get_user(&self, username: &str, url: &Url) -> DBResult<User> {
        struct InnerUser {
            profile_photo_html_url: Option<String>,
            html_url: String,
            imported: bool,
        }

        let url = db_core::clean_url(url);
        let res = sqlx::query_as!(
            InnerUser,
            "SELECT html_url, profile_photo_html_url, imported FROM starchart_users WHERE username = $1 AND 
                hostname_id = (SELECT ID FROM starchart_forges WHERE hostname = $2)",
            username,
            url,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DBError::DBError(Box::new(e)))?;
        Ok(User {
            username: username.into(),
            url,
            profile_photo: res.profile_photo_html_url,
            html_link: res.html_url,
            import: res.imported,
        })
    }

    /// check if an user exists. When url of a forge instace is provided, username search is
    /// done only on that forge
    async fn user_exists(&self, username: &str, url: Option<&Url>) -> DBResult<bool> {
        match url {
            Some(url) => {
                let url = db_core::clean_url(url);
                match sqlx::query!(
                    "SELECT ID FROM starchart_users WHERE username = $1 AND 
                hostname_id = (SELECT ID FROM starchart_forges WHERE hostname = $2)",
                    username,
                    url,
                )
                .fetch_one(&self.pool)
                .await
                {
                    Ok(_) => Ok(true),
                    Err(Error::RowNotFound) => Ok(false),
                    Err(e) => Err(DBError::DBError(Box::new(e).into())),
                }
            }
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
    async fn repository_exists(&self, name: &str, owner: &str, url: &Url) -> DBResult<bool> {
        let url = db_core::clean_url(url);
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
            url,
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
        let url = db_core::clean_url(&r.url);
        sqlx::query!(
            "INSERT INTO 
                starchart_repositories (
                    hostname_id, owner_id, name, description, html_url, website, created,
                    last_crawl, imported
                )
                VALUES (
                    (SELECT ID FROM starchart_forges WHERE hostname = $1),
                    (SELECT ID FROM starchart_users WHERE username = $2),
                    $3, $4, $5, $6, $7, $8, $9
                );",
            url,
            r.owner,
            r.name,
            r.description,
            r.html_link,
            r.website,
            now,
            now,
            r.import,
        )
        .execute(&self.pool)
        .await
        .map_err(map_register_err)?;

        self.new_fts_repositories(r.name, r.description, r.website, r.html_link)
            .await?;

        if let Some(topics) = &r.tags {
            for topic in topics.iter() {
                sqlx::query!(
                    "INSERT OR IGNORE INTO starchart_project_topics ( name ) VALUES ( $1 );",
                    topic,
                )
                .execute(&self.pool)
                .await
                .map_err(map_register_err)?;

                self.new_fts_topic(topic).await?;

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
    async fn delete_user(&self, username: &str, url: &Url) -> DBResult<()> {
        let url = db_core::clean_url(url);
        // TODO fts delete user
        sqlx::query!(
            " DELETE FROM starchart_users WHERE username = $1 AND 
                hostname_id = (SELECT ID FROM starchart_forges WHERE hostname = $2)",
            username,
            url
        )
        .execute(&self.pool)
        .await
        .map_err(map_register_err)?;
        Ok(())
    }

    /// delete repository
    async fn delete_repository(&self, owner: &str, name: &str, url: &Url) -> DBResult<()> {
        let url = db_core::clean_url(url);
        // TODO fts delete repo
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
            url
        )
        .execute(&self.pool)
        .await
        .map_err(map_register_err)?;
        Ok(())
    }

    /// Get all repositories
    async fn get_all_repositories(&self, offset: u32, limit: u32) -> DBResult<Vec<Repository>> {
        let mut db_res = sqlx::query_as!(
            InnerRepository,
            "SELECT 
                starchart_forges.hostname,
                starchart_users.username,
                starchart_repositories.name,
                starchart_repositories.description,
                starchart_repositories.html_url,
                starchart_repositories.ID,
                starchart_repositories.website,
                starchart_repositories.imported
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
            offset,
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
                url: repo.hostname,
                name: repo.name,
                username: repo.username,
                description: repo.description,
                website: repo.website,
                tags: topics,
                import: repo.imported,
            });
        }

        Ok(res)
    }

    /// Search all repositories
    async fn search_repository(&self, query: &str) -> DBResult<Vec<Repository>> {
        let mut fts_repos = self.get_fts_repository(query).await?;
        let mut res = Vec::with_capacity(fts_repos.len());
        for fts_repo in fts_repos.drain(0..) {
            let repo = sqlx::query_as!(
                InnerRepository,
                "SELECT 
                starchart_forges.hostname,
                starchart_users.username,
                starchart_repositories.name,
                starchart_repositories.description,
                starchart_repositories.html_url,
                starchart_repositories.ID,
                starchart_repositories.website,
                starchart_repositories.imported
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
            WHERE starchart_repositories.html_url =  $1
                ;",
                fts_repo.html_url
            )
            .fetch_one(&self.pool)
            .await
            .map_err(map_register_err)?;

            struct Topics {
                name: String,
            }

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
                url: repo.hostname,
                name: repo.name,
                username: repo.username,
                description: repo.description,
                website: repo.website,
                tags: topics,
                import: repo.imported,
            });
        }
        Ok(res)
    }

    /// Add Starchart instance to introducer
    async fn add_starchart_to_introducer(&self, url: &Url) -> DBResult<()> {
        let url = url.as_str();
        sqlx::query!(
            "INSERT OR IGNORE INTO
                starchart_introducer (instance_url)
            VALUES ($1);",
            url
        )
        .execute(&self.pool)
        .await
        .map_err(map_register_err)?;
        Ok(())
    }

    /// Get all introduced Starchart instances
    async fn get_all_introduced_starchart_instances(
        &self,
        offset: u32,
        limit: u32,
    ) -> DBResult<Vec<Starchart>> {
        let s = sqlx::query_as!(
            Starchart,
            "SELECT
                instance_url
            FROM
                starchart_introducer
            LIMIT $1 OFFSET $2;
        ",
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DBError::DBError(Box::new(e)))?;
        Ok(s)
    }
}

fn now_unix_time_stamp() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp()
}

struct InnerForge {
    hostname: String,
    last_crawl_on: Option<i64>,
    name: String,
    instance_url: String,
}

impl From<InnerForge> for Forge {
    fn from(f: InnerForge) -> Self {
        Self {
            url: f.hostname,
            last_crawl_on: f.last_crawl_on,
            forge_type: ForgeImplementation::from_str(&f.name).unwrap(),
            starchart_url: Some(f.instance_url),
        }
    }
}

#[allow(non_snake_case)]
struct InnerRepository {
    /// html link to the repository
    pub html_url: String,
    /// url of the forge instance: with scheme but remove trailing slash
    /// url can be derived  from html_link also, but used to link to user's forge instance
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
    pub imported: bool,
}
