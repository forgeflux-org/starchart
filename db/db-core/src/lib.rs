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
#![warn(missing_docs)]
//! # `Starchart` database operations
//!
//! Traits and datastructures used in Starchart to interact with database.
//!
//! To use an unsupported database with Starchart, traits present within this crate should be
//! implemented.
//!
//!
//! ## Organisation
//!
//! Database functionallity is divided accross various modules:
//!
//! - [errors](crate::auth): error data structures used in this crate
//! - [ops](crate::ops): meta operations like connection pool creation, migrations and getting
//! connection from pool
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use url::Url;

pub mod errors;
pub mod ops;
#[cfg(feature = "test")]
pub mod tests;

use dev::*;
pub use ops::GetConnection;

pub mod prelude {
    //! useful imports for users working with a supported database

    pub use super::errors::*;
    pub use super::ops::*;
    pub use super::*;
}

pub mod dev {
    //! useful imports for supporting a new database
    pub use super::prelude::*;
    pub use async_trait::async_trait;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// Data related to a Starchart instance
pub struct Starchart {
    /// URL of the Starchart instance
    pub instance_url: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// create a new forge on the database
pub struct CreateForge<'a> {
    /// url of the Starchart instance
    /// None = local instance
    /// Some(&'a str) = foreign instance
    pub starchart_url: Option<&'a str>,
    /// url of the forge instance: with scheme but remove trailing slash
    pub url: Url,
    /// forge type: which software is the instance running?
    pub forge_type: ForgeImplementation,
}

/// Get url from URL
/// Utility function for uniform url format
pub fn clean_url(url: &Url) -> String {
    let mut url = url.clone();
    url.set_path("");
    url.set_query(None);
    url.set_fragment(None);
    url.as_str().to_string()
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// user data
pub struct User {
    /// url of the forge instance: with scheme but remove trailing slash
    /// url can be derived  from html_link also, but used to link to user's forge instance
    pub url: String,
    /// username of the user
    pub username: String,
    /// html link to the user profile
    pub html_link: String,
    /// OPTIONAL: html link to the user's profile photo
    pub profile_photo: Option<String>,
    /// is this user an import
    pub import: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// add new user to database
pub struct AddUser<'a> {
    /// url of the forge instance: with scheme but remove trailing slash
    /// url can be derived  from html_link also, but used to link to user's forge instance
    pub url: Url,
    /// username of the user
    pub username: &'a str,
    /// html link to the user profile
    pub html_link: &'a str,
    /// OPTIONAL: html link to the user's profile photo
    pub profile_photo: Option<&'a str>,
    /// is this user an import
    pub import: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// add new repository to database
pub struct AddRepository<'a> {
    /// html link to the repository
    pub html_link: &'a str,
    /// repository topic tags
    pub tags: Option<Vec<&'a str>>,
    /// url of the forge instance: with scheme but remove trailing slash
    /// url can be derived  from html_link also, but used to link to user's forge instance
    pub url: Url,
    /// repository name
    pub name: &'a str,
    /// repository owner
    pub owner: &'a str,
    /// repository description, if any
    pub description: Option<&'a str>,
    /// repository website, if any
    pub website: Option<&'a str>,
    /// is this repository an import
    pub import: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// data representing a forge instance
pub struct Forge {
    /// url of the Starchart instance
    /// None = local instance
    /// Some(&'a str) = foreign instance
    pub starchart_url: Option<String>,
    /// url of the forge
    pub url: String,
    /// type of the forge
    pub forge_type: ForgeImplementation,
    /// last crawl
    pub last_crawl_on: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// repository
pub struct Repository {
    /// html link to the repository
    pub html_url: String,
    /// repository topic tags
    pub tags: Option<Vec<String>>,
    /// url of the forge instance: with scheme but remove trailing slash
    /// url can be derived  from html_link also, but used to link to user's forge instance
    pub url: String,
    /// repository name
    pub name: String,
    /// repository owner
    pub username: String,
    /// repository description, if any
    pub description: Option<String>,
    /// repository website, if any
    pub website: Option<String>,
    /// is this repository an import
    pub import: bool,
}

#[async_trait]
/// Starchart's database requirements. To implement support for $Database, kindly implement this
/// trait.
pub trait SCDatabase: std::marker::Send + std::marker::Sync + CloneSPDatabase {
    /// ping DB
    async fn ping(&self) -> bool;

    /// create forge instance
    async fn create_forge_instance(&self, f: &CreateForge) -> DBResult<()>;

    /// get forge instance data
    async fn get_forge(&self, url: &Url) -> DBResult<Forge>;

    /// delete forge instance
    async fn delete_forge_instance(&self, url: &Url) -> DBResult<()>;

    /// check if a forge instance exists
    async fn forge_exists(&self, url: &Url) -> DBResult<bool>;

    /// check if forge type exists
    async fn forge_type_exists(&self, forge_type: &ForgeImplementation) -> DBResult<bool>;

    /// Get all forges
    async fn get_all_forges(
        &self,
        with_imports: bool,
        offset: u32,
        limit: u32,
    ) -> DBResult<Vec<Forge>>;

    /// add new user to database
    async fn add_user(&self, u: &AddUser) -> DBResult<()>;

    /// get user data
    async fn get_user(&self, username: &str, url: &Url) -> DBResult<User>;

    /// check if an user exists. When url of a forge instance is provided, username search is
    /// done only on that forge
    async fn user_exists(&self, username: &str, url: Option<&Url>) -> DBResult<bool>;

    /// delete user
    async fn delete_user(&self, username: &str, url: &Url) -> DBResult<()>;

    /// delete repository
    async fn delete_repository(&self, owner: &str, name: &str, url: &Url) -> DBResult<()>;

    /// check if a repository exists.
    async fn repository_exists(&self, name: &str, owner: &str, url: &Url) -> DBResult<bool>;

    /// Get all repositories
    async fn get_all_repositories(&self, offset: u32, limit: u32) -> DBResult<Vec<Repository>>;

    /// add new repository to database.
    async fn create_repository(&self, r: &AddRepository) -> DBResult<()>;

    /// Search all repositories
    async fn search_repository(&self, query: &str) -> DBResult<Vec<Repository>>;

    /// Add Starchart instance to introducer
    async fn add_starchart_to_introducer(&self, url: &Url) -> DBResult<()>;

    /// Get all introduced Starchart instances
    async fn get_all_introduced_starchart_instances(
        &self,
        offset: u32,
        limit: u32,
    ) -> DBResult<Vec<Starchart>>;

    /// Add word to mini index
    async fn add_word_to_mini_index(&self, word: &str) -> DBResult<()>;

    /// Remove word from mini index
    async fn rm_word_from_mini_index(&self, word: &str) -> DBResult<()>;

    /// Check if word exists in mini index
    async fn is_word_mini_indexed(&self, word: &str) -> DBResult<bool>;

    /// consolidate and export mini index
    async fn export_mini_index(&self) -> DBResult<String>;

    /// Import mini-index
    async fn import_mini_index(
        &self,
        starchart_instance_url: &Url,
        mini_index: &str,
    ) -> DBResult<()>;

    /// Delete imported mini-index
    async fn rm_imported_mini_index(&self, starchart_instance_url: &Url) -> DBResult<()>;

    /// Search mini index
    async fn search_mini_index(&self, query: &str) -> DBResult<Vec<String>>;

    /// Mark a Starchart instance as imported
    async fn record_starchart_imports(&self, starchart_url: &Url) -> DBResult<()>;

    /// Unmark a Starchart instance as imported
    async fn rm_starchart_import(&self, starchart_url: &Url) -> DBResult<()>;

    /// Check if Starchart instance is imported
    async fn is_starchart_imported(&self, starchart_url: &Url) -> DBResult<bool>;
}

/// Trait to clone SCDatabase
pub trait CloneSPDatabase {
    /// clone DB
    fn clone_db(&self) -> Box<dyn SCDatabase>;
}

impl<T> CloneSPDatabase for T
where
    T: SCDatabase + Clone + 'static,
{
    fn clone_db(&self) -> Box<dyn SCDatabase> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn SCDatabase> {
    fn clone(&self) -> Self {
        (**self).clone_db()
    }
}

/// Forge type: Gitea, Sourcehut, GitLab, etc. Support is currently only available for Gitea
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ForgeImplementation {
    /// [Gitea](https://gitea.io) softare forge
    Gitea,
}

impl ForgeImplementation {
    /// Convert [ForgeImplementation] to [str]
    pub const fn to_str(&self) -> &'static str {
        match self {
            ForgeImplementation::Gitea => "gitea",
        }
    }
}

impl FromStr for ForgeImplementation {
    type Err = DBError;

    /// Convert [str] to [ForgeImplementation]
    fn from_str(s: &str) -> DBResult<Self> {
        const GITEA: &str = ForgeImplementation::Gitea.to_str();
        let s = s.trim();
        match s {
            GITEA => Ok(Self::Gitea),
            _ => Err(DBError::UnknownForgeType(s.to_owned())),
        }
    }
}
