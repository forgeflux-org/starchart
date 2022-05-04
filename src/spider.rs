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
use std::time::Duration;

use tokio::time;
use url::Url;

use db_core::prelude::*;

use crate::data::Data;
use crate::db::BoxDB;
use crate::gitea::SearchResults;
use crate::gitea::Topics;

const REPO_SEARCH_PATH: &str = "/api/v1/repos/search";
const GITEA_NODEINFO: &str = "/api/v1/nodeinfo";

impl Data {
    pub async fn crawl(&self, hostname: &str, db: &BoxDB) -> Vec<SearchResults> {
        fn empty_is_none(s: &str) -> Option<&str> {
            if s.trim().is_empty() {
                None
            } else {
                Some(s)
            }
        }

        let mut page = 1;
        let instance_url = Url::parse(hostname).unwrap();
        let hostname = get_hostname(&instance_url);
        if !db.forge_exists(&hostname).await.unwrap() {
            let msg = CreateForge {
                hostname: &hostname,
                forge_type: ForgeImplementation::Gitea,
            };
            db.create_forge_isntance(&msg).await.unwrap();
        }

        let mut url = instance_url.clone();
        url.set_path(REPO_SEARCH_PATH);
        let mut repos = Vec::new();
        loop {
            let mut url = url.clone();
            url.set_query(Some(&format!(
                "page={page}&limit={}",
                self.settings.crawler.items_per_api_call
            )));
            let res: SearchResults = self
                .client
                .get(url)
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();

            let sleep_fut = time::sleep(Duration::new(
                self.settings.crawler.wait_before_next_api_call,
                0,
            ));
            let sleep_fut = tokio::spawn(sleep_fut);

            for repo in res.data.iter() {
                if !db
                    .user_exists(&repo.owner.username, Some(&hostname))
                    .await
                    .unwrap()
                {
                    let mut profile_url = instance_url.clone();
                    profile_url.set_path(&repo.owner.username);
                    let msg = AddUser {
                        hostname: &hostname,
                        username: &repo.owner.username,
                        html_link: profile_url.as_str(),
                        profile_photo: Some(&repo.owner.avatar_url),
                    };
                    db.add_user(&msg).await.unwrap();
                }

                let mut url = instance_url.clone();
                url.set_path(&format!(
                    "/api/v1/repos/{}/{}/topics",
                    repo.owner.username, repo.name
                ));

                let topics: Topics = self
                    .client
                    .get(url)
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();

                let add_repo_msg = AddRepository {
                    tags: Some(topics.topics),
                    name: &repo.name,
                    website: empty_is_none(&repo.website),
                    description: empty_is_none(&repo.description),
                    owner: &repo.owner.username,
                    html_link: &repo.html_url,
                    hostname: &hostname,
                };

                db.create_repository(&add_repo_msg).await.unwrap();
            }

            sleep_fut.await.unwrap();
            if res.data.is_empty() {
                return repos;
            }
            repos.push(res);
            page += 1;
        }
    }

    /// purpose: interact with instance running on provided hostname and verify if the instance is a
    /// Gitea instance.
    ///
    /// will get nodeinfo information, which contains an identifier to uniquely identify Gitea
    pub async fn is_gitea(&self, hostname: &str) -> bool {
        const GITEA_IDENTIFIER: &str = "gitea";
        let mut url = Url::parse(hostname).unwrap();
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
}

#[cfg(test)]
mod tests {
    use crate::tests::sqlx_sqlite;
    use db_core::prelude::*;

    use url::Url;

    pub const GITEA_HOST: &str = "http://localhost:8080";

    #[actix_rt::test]
    async fn is_gitea_works() {
        let (_db, data) = sqlx_sqlite::get_data().await;
        assert!(data.is_gitea(GITEA_HOST).await);
    }

    #[actix_rt::test]
    async fn crawl_gitea() {
        let (db, data) = sqlx_sqlite::get_data().await;
        let res = data.crawl(GITEA_HOST, &db).await;
        let mut elements = 0;
        let username = &res.get(0).unwrap().data.get(0).unwrap().owner.username;
        let hostname = get_hostname(&Url::parse(GITEA_HOST).unwrap());
        assert!(db.forge_exists(&hostname).await.unwrap());
        assert!(db.user_exists(username, Some(&hostname)).await.unwrap());
        res.iter().for_each(|r| elements += r.data.len());

        assert_eq!(res.len(), 5);
        assert_eq!(elements, 100);
    }
}
