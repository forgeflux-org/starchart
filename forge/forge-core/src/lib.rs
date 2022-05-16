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
pub struct User<'a> {
    /// hostname of the forge instance: with scheme but remove trailing slash
    /// hostname can be derived  from html_link also, but used to link to user's forge instance
    pub hostname: &'a str,
    /// username of the user
    pub username: Arc<String>,
    /// html link to the user profile
    pub html_link: String,
    /// OPTIONAL: html link to the user's profile photo
    pub profile_photo: Option<String>,
}

impl<'a> From<&'a User<'a>> for AddUser<'a> {
    fn from(u: &'a User) -> Self {
        Self {
            hostname: u.hostname,
            username: u.username.as_str(),
            html_link: &u.html_link,
            profile_photo: u.profile_photo.as_deref(),
        }
    }
}

#[derive(Clone, Debug)]
/// add new repository to database
pub struct Repository<'a> {
    /// html link to the repository
    pub html_link: String,
    /// repository topic tags
    pub tags: Option<Vec<Arc<String>>>,
    /// hostname of the forge instance: with scheme but remove trailing slash
    /// hostname can be deras_ref().map(|p| p.as_str()),ived  from html_link also, but used to link to user's forge instance
    pub hostname: &'a str,
    /// repository name
    pub name: String,
    /// repository owner
    pub owner: Arc<User<'a>>,
    /// repository description, if any
    pub description: Option<String>,
    /// repository website, if any
    pub website: Option<String>,
}

impl<'a> From<&'a Repository<'a>> for AddRepository<'a> {
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
            hostname: r.hostname,
            name: &r.name,
            description: r.description.as_deref(),
            owner: r.owner.username.as_str(),
            tags,
            html_link: &r.html_link,
            website: r.website.as_deref(),
        }
    }
}

pub type UserMap<'a> = HashMap<Arc<String>, Arc<User<'a>>>;
pub type Tags = HashSet<Arc<String>>;
pub type Repositories<'a> = Vec<Repository<'a>>;

pub struct CrawlResp<'a> {
    pub repos: Repositories<'a>,
    pub tags: Tags,
    pub users: UserMap<'a>,
}

#[async_trait]
pub trait SCForge: std::marker::Send + std::marker::Sync + CloneSPForge {
    async fn is_forge(&self) -> bool;
    async fn crawl(&self, limit: u64, page: u64) -> CrawlResp;
    fn get_hostname(&self) -> &str;
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
