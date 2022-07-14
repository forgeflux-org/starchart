/*
 * ForgeFlux StarChart - A federated software forge spider
 * Copyright © 2022 Aravinth Manivannan <realaravinth@batsense.net>
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
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use async_trait::async_trait;
use db_core::prelude::*;
use url::Url;

pub mod prelude {
    pub use super::*;
    pub use async_trait::async_trait;
}

pub mod dev {
    pub use super::*;
    pub use async_trait::async_trait;
    pub use db_core;
}

#[derive(Clone, Debug)]
pub struct User {
    /// url of the forge instance: with scheme but remove trailing slash
    /// url can be derived  from html_link also, but used to link to user's forge instance
    pub url: Url,
    /// username of the user
    pub username: Arc<String>,
    /// html link to the user profile
    pub html_link: String,
    /// OPTIONAL: html link to the user's profile photo
    pub profile_photo: Option<String>,
}

impl<'a> From<&'a User> for AddUser<'a> {
    fn from(u: &'a User) -> Self {
        Self {
            url: u.url.clone(),
            username: u.username.as_str(),
            html_link: &u.html_link,
            profile_photo: u.profile_photo.as_deref(),
        }
    }
}

#[derive(Clone, Debug)]
/// add new repository to database
pub struct Repository {
    /// html link to the repository
    pub html_link: String,
    /// repository topic tags
    pub tags: Option<Vec<Arc<String>>>,
    /// url of the forge instance: with scheme but remove trailing slash
    /// url can be deras_ref().map(|p| p.as_str()),ived  from html_link also, but used to link to user's forge instance
    pub url: Url,
    /// repository name
    pub name: String,
    /// repository owner
    pub owner: Arc<User>,
    /// repository description, if any
    pub description: Option<String>,
    /// repository website, if any
    pub website: Option<String>,
}

impl<'a> From<&'a Repository> for AddRepository<'a> {
    fn from(r: &'a Repository) -> Self {
        let tags = if let Some(rtags) = &r.tags {
            let mut tags = Vec::with_capacity(rtags.len());
            for t in rtags.iter() {
                tags.push(t.as_str());
            }
            Some(tags)
        } else {
            None
        };
        Self {
            url: r.url.clone(),
            name: &r.name,
            description: r.description.as_deref(),
            owner: r.owner.username.as_str(),
            tags,
            html_link: &r.html_link,
            website: r.website.as_deref(),
        }
    }
}

pub type UserMap = HashMap<Arc<String>, Arc<User>>;
pub type Tags = HashSet<Arc<String>>;
pub type Repositories = Vec<Repository>;

pub struct CrawlResp {
    pub repos: Repositories,
    pub tags: Tags,
    pub users: UserMap,
}

#[async_trait]
pub trait SCForge: std::marker::Send + std::marker::Sync + CloneSPForge {
    async fn is_forge(&self) -> bool;
    async fn crawl(&self, limit: u64, page: u64, rate_limit: u64) -> CrawlResp;
    fn get_url(&self) -> &Url;
    fn forge_type(&self) -> ForgeImplementation;
}

/// Trait to clone SCForge
pub trait CloneSPForge {
    /// clone Forge
    fn clone_forge(&self) -> Box<dyn SCForge>;
}

impl<T> CloneSPForge for T
where
    T: SCForge + Clone + 'static,
{
    fn clone_forge(&self) -> Box<dyn SCForge> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn SCForge> {
    fn clone(&self) -> Self {
        (**self).clone_forge()
    }
}
