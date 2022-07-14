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
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use serde::Serialize;
use tokio::fs;
use url::Url;

use db_core::prelude::*;

use federate_core::Federate;

pub mod errors;
pub mod schema;
#[cfg(test)]
mod tests;

use errors::*;

pub const INSTANCE_INFO_FILE: &str = "instance.yml";
pub const USER_INFO_FILE: &str = "user.yml";
pub const REPO_INFO_FILE: &str = "publiccode.yml";

pub const CONTENTS_DIR: &str = "uncompressed";

#[derive(Clone)]
pub struct PccFederate {
    pub base_dir: String,
}

impl PccFederate {
    pub async fn new(base_dir: String) -> FResult<Self> {
        let x = Self { base_dir };
        x.get_content_path(true).await?;
        Ok(x)
    }

    pub async fn get_content_path(&self, create_dirs: bool) -> FResult<PathBuf> {
        let path = Path::new(&self.base_dir).join(CONTENTS_DIR);
        if create_dirs {
            self.create_dir_if_not_exists(&path).await?;
        }
        Ok(path)
    }

    pub async fn get_instance_path(&self, url: &Url, create_dirs: bool) -> FResult<PathBuf> {
        let hostname = federate_core::get_hostname(url);
        let path = self.get_content_path(false).await?.join(hostname);
        if create_dirs {
            self.create_dir_if_not_exists(&path).await?;
        }
        Ok(path)
    }

    pub async fn get_user_path(
        &self,
        username: &str,
        url: &Url,
        create_dirs: bool,
    ) -> FResult<PathBuf> {
        let path = self.get_instance_path(url, false).await?.join(username);
        if create_dirs {
            self.create_dir_if_not_exists(&path).await?;
        }
        Ok(path)
    }

    pub async fn get_repo_path(
        &self,
        name: &str,
        owner: &str,
        url: &Url,
        create_dirs: bool,
    ) -> FResult<PathBuf> {
        let path = self.get_user_path(owner, url, false).await?.join(name);
        if create_dirs {
            self.create_dir_if_not_exists(&path).await?;
        }
        Ok(path)
    }

    /// utility method to write data
    async fn write_util<S: Serialize + Send + Sync>(&self, data: &S, path: &Path) -> FResult<()> {
        let fcontents = serde_yaml::to_string(data)?;
        fs::write(path, &fcontents).await?;
        Ok(())
    }
}

#[async_trait]
impl Federate for PccFederate {
    type Error = FederateErorr;

    /// utility method to create dir if not exists
    async fn create_dir_if_not_exists(&self, path: &Path) -> FResult<()> {
        if !path.exists() {
            fs::create_dir_all(path).await?;
        }
        Ok(())
    }

    /// utility method to remove file/dir
    async fn rm_util(&self, path: &Path) -> FResult<()> {
        if path.exists() {
            if path.is_dir() {
                fs::remove_dir_all(path).await?;
            } else {
                fs::remove_file(&path).await?;
            }
        }
        Ok(())
    }

    /// create forge instance
    async fn create_forge_instance(&self, f: &CreateForge) -> FResult<()> {
        let path = self.get_instance_path(&f.url, true).await?;
        self.write_util(f, &path.join(INSTANCE_INFO_FILE)).await?;
        Ok(())
    }

    /// delete forge instance
    async fn delete_forge_instance(&self, url: &Url) -> FResult<()> {
        let path = self.get_instance_path(&url, false).await?;
        self.rm_util(&path).await
    }

    /// check if a forge instance exists
    async fn forge_exists(&self, url: &Url) -> Result<bool, Self::Error> {
        let path = self.get_instance_path(url, false).await?;
        if path.exists() && path.is_dir() {
            let instance = path.join(INSTANCE_INFO_FILE);
            Ok(instance.exists() && instance.is_file())
        } else {
            Ok(false)
        }
    }

    /// check if an user exists.
    async fn user_exists(&self, username: &str, url: &Url) -> Result<bool, Self::Error> {
        let path = self.get_user_path(username, url, false).await?;
        if path.exists() && path.is_dir() {
            let user = path.join(USER_INFO_FILE);
            Ok(user.exists() && user.is_file())
        } else {
            Ok(false)
        }
    }

    /// create user instance
    async fn create_user(&self, f: &AddUser<'_>) -> Result<(), Self::Error> {
        let path = self.get_user_path(f.username, &f.url, true).await?;
        self.write_util(f, &path.join(USER_INFO_FILE)).await
    }

    /// add repository instance
    async fn create_repository(&self, f: &AddRepository<'_>) -> Result<(), Self::Error> {
        let path = self
            .get_repo_path(f.name, f.owner, &f.url, true)
            .await?
            .join(REPO_INFO_FILE);
        let publiccode: schema::Repository = f.into();
        self.write_util(&publiccode, &path).await
    }

    /// check if a repository exists.
    async fn repository_exists(
        &self,
        name: &str,
        owner: &str,
        url: &Url,
    ) -> Result<bool, Self::Error> {
        let path = self.get_repo_path(name, owner, url, false).await?;
        if path.exists() && path.is_dir() {
            let repo = path.join(REPO_INFO_FILE);
            Ok(repo.exists() && repo.is_file())
        } else {
            Ok(false)
        }
    }

    /// delete user
    async fn delete_user(&self, username: &str, url: &Url) -> Result<(), Self::Error> {
        let path = self.get_user_path(username, url, false).await?;
        self.rm_util(&path).await?;
        Ok(())
    }

    /// delete repository
    async fn delete_repository(
        &self,
        owner: &str,
        name: &str,
        url: &Url,
    ) -> Result<(), Self::Error> {
        let path = self.get_repo_path(name, owner, url, false).await?;
        self.rm_util(&path).await
    }

    async fn tar(&self) -> Result<PathBuf, Self::Error> {
        use std::fs::File;
        use std::time::{SystemTime, UNIX_EPOCH};

        use tar::Builder;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let path = Path::new(&self.base_dir).join(format!("{now}.tar"));
        let file = File::create(&path)?;
        let mut a = Builder::new(file);
        a.append_dir_all(".", self.get_content_path(false).await?)
            .unwrap();
        a.finish().unwrap();
        Ok(path)
    }
}
