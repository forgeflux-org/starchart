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

const REPO_SEARCH_PATH: &str = "/api/v1/repos/search";
const GITEA_NODEINFO: &str = "/api/v1/nodeinfo";

impl Data {
    pub async fn crawl(&self, hostname: &str, db: &BoxDB) -> Vec<SearchResults> {
        let mut page = 1;
        let mut url = Url::parse(hostname).unwrap();
        let hostname = url.host().as_ref().unwrap().to_string();
        if !db.forge_exists(&hostname).await.unwrap() {
            let msg = CreateForge {
                hostname: &hostname,
                forge_type: ForgeImplementation::Gitea,
            };
            db.create_forge_isntance(&msg).await.unwrap();
        }

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

            time::sleep(Duration::new(
                self.settings.crawler.wait_before_next_api_call,
                0,
            ))
            .await;
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
        res.iter().for_each(|r| elements += r.data.len());
        assert_eq!(res.len(), 5);
        assert_eq!(elements, 100);
    }
}
