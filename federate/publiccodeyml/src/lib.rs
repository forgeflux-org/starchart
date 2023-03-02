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
use std::fs as StdFs;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use log::info;
use mktemp::Temp;
use reqwest::Client;
use serde::Serialize;
use tar::Archive;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use url::Url;

use db_core::prelude::*;

use federate_core::Federate;
use federate_core::{LatestResp, ROUTES};

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
        let path = self.get_instance_path(url, false).await?;
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

        let mut times: Vec<usize> = Vec::with_capacity(10);
        let mut dir = fs::read_dir(Path::new(&self.base_dir)).await?;
        while let Some(d) = dir.next_entry().await? {
            if d.path().is_dir() {
                continue;
            }
            let file = d.file_name().into_string().unwrap();
            if file.ends_with(".tar") {
                if let Some(time) = file.split(".tar").next() {
                    times.push(time.parse::<usize>().unwrap());
                }
            }
        }

        times.sort();

        let mut times = times.iter().rev();
        for _ in 0..5 {
            times.next();
        }
        for t in times {
            let file = Path::new(&self.base_dir).join(format!("{t}.tar"));
            fs::remove_file(file).await?;
        }

        Ok(path)
    }

    /// get latest tar ball
    async fn latest_tar(&self) -> Result<String, Self::Error> {
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

        let mut times: Vec<usize> = Vec::with_capacity(10);
        let mut dir = fs::read_dir(Path::new(&self.base_dir)).await?;
        while let Some(d) = dir.next_entry().await? {
            if d.path().is_dir() {
                continue;
            }
            let file = d.file_name().into_string().unwrap();
            if file.ends_with(".tar") {
                if let Some(time) = file.split(".tar").next() {
                    times.push(time.parse::<usize>().unwrap());
                }
            }
        }

        times.sort();

        let latest = times.pop().unwrap();
        Ok(format!("{}.tar", latest))
    }

    /// import archive from another Starchart instance
    async fn import(
        &self,
        starchart_url: Url,
        client: &Client,
        db: &Box<dyn SCDatabase>,
    ) -> Result<(), Self::Error> {
        info!("[import][{starchart_url}] import latest tarball from starchart instance");

        let mut url = starchart_url.clone();
        url.set_path(ROUTES.get_latest);
        let resp: LatestResp = client.get(url).send().await.unwrap().json().await.unwrap();
        let mut url = starchart_url.clone();
        url.set_path(&format!("/federate/{}", resp.latest));
        println!("{:?}", url);
        let file = client.get(url).send().await.unwrap().bytes().await.unwrap();
        let tmp = Temp::new_dir().unwrap();
        let import_file = tmp.as_path().join("import.tar.gz");
        {
            let mut f = fs::File::create(&import_file).await.unwrap();
            f.write_all(&file).await.unwrap();
        }

        let f = StdFs::File::open(&import_file).unwrap();
        let uncompressed = tmp.as_path().join("untar");
        fs::create_dir(&uncompressed).await.unwrap();

        let mut ar = Archive::new(f);
        ar.unpack(&uncompressed).unwrap();

        let mut instance_dir_contents = fs::read_dir(&uncompressed).await.unwrap();
        while let Some(instance_dir_entry) = instance_dir_contents.next_entry().await.unwrap() {
            if !instance_dir_entry.file_type().await.unwrap().is_dir() {
                continue;
            }

            let instance_file = instance_dir_entry.path().join(INSTANCE_INFO_FILE);
            let instance = fs::read_to_string(instance_file).await.unwrap();
            let mut instance: CreateForge = serde_yaml::from_str(&instance).unwrap();
            instance.starchart_url = Some(starchart_url.as_str());

            if !db.forge_exists(&instance.url).await.unwrap() {
                info!("[import][{}] Creating forge", &instance.url);

                db.create_forge_instance(&instance).await.unwrap();
            } else if !self.forge_exists(&instance.url).await.unwrap() {
                self.create_forge_instance(&instance).await.unwrap();
            }

            let mut dir_contents = fs::read_dir(&instance_dir_entry.path()).await.unwrap();
            while let Some(dir_entry) = dir_contents.next_entry().await.unwrap() {
                if !dir_entry.file_type().await.unwrap().is_dir() {
                    continue;
                }
                let username = dir_entry.file_name();
                let username = username.to_str().unwrap();

                if !db.user_exists(username, Some(&instance.url)).await.unwrap() {
                    info!("[import][{}] Creating user: {username}", instance.url);

                    let user_file = instance_dir_entry
                        .path()
                        .join(username)
                        .join(USER_INFO_FILE);
                    let user_file_content = fs::read_to_string(user_file).await.unwrap();
                    let mut user: AddUser<'_> = serde_yaml::from_str(&user_file_content).unwrap();
                    user.import = true;

                    db.add_user(&user).await.unwrap();
                }
                if !self.user_exists(username, &instance.url).await.unwrap() {
                    let user_file = instance_dir_entry
                        .path()
                        .join(username)
                        .join(USER_INFO_FILE);
                    let user_file_content = fs::read_to_string(user_file).await.unwrap();
                    let mut user: AddUser<'_> = serde_yaml::from_str(&user_file_content).unwrap();
                    user.import = true;

                    self.create_user(&user).await.unwrap();
                }

                let mut repositories = fs::read_dir(dir_entry.path()).await.unwrap();
                while let Some(repo) = repositories.next_entry().await.unwrap() {
                    if !repo.file_type().await.unwrap().is_dir() {
                        continue;
                    }
                    let repo_file = repo.path().join(REPO_INFO_FILE);
                    println!("repo_file: {:?}", repo_file);
                    let publiccodeyml_repository: schema::Repository =
                        serde_yaml::from_str(&fs::read_to_string(repo_file).await.unwrap())
                            .unwrap();
                    let add_repo = publiccodeyml_repository.to_add_repository(true);

                    if !db
                        .repository_exists(add_repo.name, username, &add_repo.url)
                        .await
                        .unwrap()
                    {
                        info!(
                            "[import][{}] Creating repository: {}",
                            instance.url, add_repo.name
                        );
                        db.create_repository(&add_repo).await.unwrap();
                    }
                    if !self
                        .repository_exists(add_repo.name, username, &add_repo.url)
                        .await
                        .unwrap()
                    {
                        self.create_repository(&add_repo).await.unwrap();
                    }
                }
            }
        }
        Ok(())
    }
}
