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

use db_core::prelude::*;

#[cfg(feature = "test")]
pub mod tests;

#[async_trait]
pub trait Federate: Sync + Send {
    type Error: std::error::Error + std::fmt::Debug;

    /// utility method to create dir if not exists
    async fn create_dir_if_not_exists(&self, path: &Path) -> Result<(), Self::Error>;

    /// utility method to remove file/dir
    async fn rm_util(&self, path: &Path) -> Result<(), Self::Error>;

    /// create forge isntance
    async fn create_forge_isntance(&self, f: &CreateForge<'_>) -> Result<(), Self::Error>;

    /// delete forge isntance
    async fn delete_forge_instance(&self, hostname: &str) -> Result<(), Self::Error>;

    /// check if a forge instance exists
    async fn forge_exists(&self, hostname: &str) -> Result<bool, Self::Error>;

    /// create user isntance
    async fn create_user(&self, f: &AddUser<'_>) -> Result<(), Self::Error>;

    /// add repository isntance
    async fn create_repository(&self, f: &AddRepository<'_>) -> Result<(), Self::Error>;

    /// delete user
    async fn delete_user(&self, username: &str, hostname: &str) -> Result<(), Self::Error>;

    /// delete repository
    async fn delete_repository(
        &self,
        owner: &str,
        name: &str,
        hostname: &str,
    ) -> Result<(), Self::Error>;

    /// publish results in tar ball
    async fn tar(&self) -> Result<PathBuf, Self::Error>;
}
