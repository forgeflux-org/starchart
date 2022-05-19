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
use std::sync::Arc;

use reqwest::Client;
use url::Url;

use db_core::ForgeImplementation;
use db_core::SCDatabase;
use forge_core::dev::*;
use forge_core::Repository;

pub mod schema;

const REPO_SEARCH_PATH: &str = "/api/v1/repos/search";
const GITEA_NODEINFO: &str = "/api/v1/nodeinfo";
const GITEA_IDENTIFIER: &str = "gitea";

#[derive(Clone)]
pub struct Gitea {
    pub instance_url: Url,
    pub client: Client,
    hostname: String,
}

impl Gitea {
    pub fn new(instance_url: Url, client: Client) -> Self {
        let hostname = db_core::get_hostname(&instance_url);

        Self {
            instance_url,
            client,
            hostname,
        }
    }
}

impl PartialEq for Gitea {
    fn eq(&self, other: &Self) -> bool {
        self.hostname == other.hostname && self.instance_url == other.instance_url
    }
}

#[async_trait]
impl SCForge for Gitea {
    async fn is_forge(&self) -> bool {
        let mut url = self.instance_url.clone();
        url.set_path(GITEA_NODEINFO);

        let res: serde_json::Value = self
            .client
            .get(url)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        if let serde_json::Value::String(software) = &res["software"]["name"] {
            software == GITEA_IDENTIFIER
        } else {
            false
        }
    }

    fn get_hostname(&self) -> &str {
        &self.hostname
    }

    fn forge_type(&self) -> ForgeImplementation {
        ForgeImplementation::Gitea
    }

    async fn crawl(&self, limit: u64, page: u64) -> CrawlResp {
        fn empty_is_none(s: &str) -> Option<String> {
            let s = s.trim();
            if s.is_empty() {
                None
            } else {
                Some(s.to_owned())
            }
        }

        let mut tags = Tags::default();
        let mut users = UserMap::default();
        let mut repos = Repositories::default();

        let instance_url = self.instance_url.clone();

        let mut url = instance_url.clone();
        url.set_path(REPO_SEARCH_PATH);
        url.set_query(Some(&format!("page={page}&limit={limit}")));
        let mut res: schema::SearchResults = self
            .client
            .get(url)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        fn to_user(u: schema::User, g: &Gitea) -> Arc<User<'_>> {
            let mut profile_url = g.instance_url.clone();
            profile_url.set_path(&u.username);
            let username = Arc::new(u.username);
            Arc::new(User {
                username,
                html_link: profile_url.to_string(),
                profile_photo: Some(u.avatar_url),
                hostname: &g.hostname,
            })
        }

        for repo in res.data.drain(0..) {
            let user = if !users.contains_key(&repo.owner.username) {
                let u = to_user(repo.owner, self);
                let username = u.username.clone();
                users.insert(username.clone().clone(), u.clone());
                u
            } else {
                users.get(&repo.owner.username).unwrap().clone()
            };

            let mut url = instance_url.clone();
            url.set_path(&format!(
                "/api/v1/repos/{}/{}/topics",
                &user.username, repo.name
            ));

            let mut topics: schema::Topics = self
                .client
                .get(url)
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();

            let mut rtopics = Vec::with_capacity(topics.topics.len());
            for t in topics.topics.drain(0..) {
                let t = Arc::new(t);
                if !tags.contains(&t) {
                    tags.insert(t.clone());
                }
                rtopics.push(t);
            }

            let frepo = Repository {
                hostname: &self.hostname,
                website: empty_is_none(&repo.website),
                name: repo.name,
                owner: user,
                html_link: repo.html_url,
                tags: Some(rtopics),
                description: Some(repo.description),
            };

            repos.push(frepo);
        }
        CrawlResp { repos, tags, users }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    pub const GITEA_HOST: &str = "http://localhost:8080";
    pub const NET_REPOSITORIES: u64 = 100;
    pub const PER_CRAWL: u64 = 10;

    #[actix_rt::test]
    async fn gitea_works() {
        let ctx = Gitea::new(Url::parse(GITEA_HOST).unwrap(), Client::new());
        assert!(ctx.is_forge().await);
        let steps = NET_REPOSITORIES / PER_CRAWL;

        for i in 0..steps {
            let res = ctx.crawl(PER_CRAWL, i).await;
            assert_eq!(res.repos.len() as u64, PER_CRAWL);
        }
    }
}
