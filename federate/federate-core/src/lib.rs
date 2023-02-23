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
use std::path::Path;
use std::path::PathBuf;
use std::result::Result;

use async_trait::async_trait;
use reqwest::Client;
use url::Url;

use db_core::prelude::*;

#[cfg(feature = "test")]
pub mod tests;

pub use api_routes::*;

#[async_trait]
pub trait Federate: Sync + Send {
    type Error: std::error::Error + std::fmt::Debug;

    /// utility method to create dir if not exists
    async fn create_dir_if_not_exists(&self, path: &Path) -> Result<(), Self::Error>;

    /// utility method to remove file/dir
    async fn rm_util(&self, path: &Path) -> Result<(), Self::Error>;

    /// create forge instance
    async fn create_forge_instance(&self, f: &CreateForge) -> Result<(), Self::Error>;

    /// delete forge instance
    async fn delete_forge_instance(&self, url: &Url) -> Result<(), Self::Error>;

    /// check if a forge instance exists
    async fn forge_exists(&self, url: &Url) -> Result<bool, Self::Error>;

    /// check if an user exists.
    async fn user_exists(&self, username: &str, url: &Url) -> Result<bool, Self::Error>;

    /// create user instance
    async fn create_user(&self, f: &AddUser<'_>) -> Result<(), Self::Error>;

    /// add repository instance
    async fn create_repository(&self, f: &AddRepository<'_>) -> Result<(), Self::Error>;

    /// check if a repository exists.
    async fn repository_exists(
        &self,
        name: &str,
        owner: &str,
        url: &Url,
    ) -> Result<bool, Self::Error>;

    /// delete user
    async fn delete_user(&self, username: &str, url: &Url) -> Result<(), Self::Error>;

    /// delete repository
    async fn delete_repository(
        &self,
        owner: &str,
        name: &str,
        url: &Url,
    ) -> Result<(), Self::Error>;

    /// publish results in tar ball
    async fn tar(&self) -> Result<PathBuf, Self::Error>;

    /// get latest tar ball
    async fn latest_tar(&self) -> Result<String, Self::Error>;

    /// import archive from another Starchart instance
    async fn import(
        &self,
        mut starchart_url: Url,
        client: &Client,
        db: &Box<dyn SCDatabase>,
    ) -> Result<(), Self::Error>;

    async fn latest_tar_json(&self) -> Result<LatestResp, Self::Error> {
        let latest = self.latest_tar().await?;
        Ok(LatestResp { latest })
    }
}

pub fn get_hostname(url: &Url) -> &str {
    url.host_str().unwrap()
}
