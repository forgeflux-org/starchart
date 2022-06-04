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
use log::info;
use url::Url;

use db_core::prelude::*;
use forge_core::prelude::*;
use gitea::Gitea;

use crate::ctx::Ctx;
use crate::db::BoxDB;
use crate::federate::ArcFederate;

impl Ctx {
    pub async fn crawl(&self, instance_url: &str, db: &BoxDB, federate: &ArcFederate) {
        let gitea = Gitea::new(Url::parse(instance_url).unwrap(), self.client.clone());
        let mut page = 1;
        let hostname = gitea.get_hostname();
        if !db.forge_exists(hostname).await.unwrap() {
            info!("[crawl][{hostname}] Creating forge");
            let msg = CreateForge {
                hostname,
                forge_type: gitea.forge_type(),
            };
            db.create_forge_isntance(&msg).await.unwrap();
        } else {
            if !federate.forge_exists(hostname).await.unwrap() {
                let forge = db.get_forge(hostname).await.unwrap();
                let msg = CreateForge {
                    hostname,
                    forge_type: forge.forge_type,
                };
                federate.create_forge_isntance(&msg).await.unwrap();
            }
        }

        loop {
            info!("[crawl][{hostname}] Crawling. page: {page}");
            let res = gitea
                .crawl(
                    self.settings.crawler.items_per_api_call,
                    page,
                    self.settings.crawler.wait_before_next_api_call,
                )
                .await;
            if res.repos.is_empty() {
                info!("[crawl][{hostname}] Finished crawling. pages: {}", page - 1);
                break;
            }

            for (username, u) in res.users.iter() {
                if !db
                    .user_exists(username, Some(gitea.get_hostname()))
                    .await
                    .unwrap()
                {
                    info!("[crawl][{hostname}] Creating user: {username}");
                    let msg = u.as_ref().into();
                    db.add_user(&msg).await.unwrap();
                    federate.create_user(&msg).await.unwrap();
                }
            }

            for r in res.repos.iter() {
                if !db
                    .repository_exists(&r.name, &r.owner.username, r.hostname)
                    .await
                    .unwrap()
                {
                    info!("[crawl][{hostname}] Creating repository: {}", r.name);
                    let msg = r.into();
                    db.create_repository(&msg).await.unwrap();
                    federate.create_repository(&msg).await.unwrap();
                }
            }

            //            sleep_fut.await.unwrap();
            page += 1;
        }
        federate.tar().await.unwrap();
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::sqlx_sqlite;
    use db_core::prelude::*;

    use url::Url;

    pub const GITEA_HOST: &str = "http://localhost:8080";
    pub const GITEA_USERNAME: &str = "bot";

    #[actix_rt::test]
    async fn crawl_gitea() {
        let (db, ctx, federate, _tmp_dir) = sqlx_sqlite::get_ctx().await;
        ctx.crawl(GITEA_HOST, &db, &federate).await;
        let hostname = get_hostname(&Url::parse(GITEA_HOST).unwrap());
        assert!(db.forge_exists(&hostname).await.unwrap());
        assert!(db
            .user_exists(GITEA_USERNAME, Some(&hostname))
            .await
            .unwrap());
        assert!(db.user_exists(GITEA_USERNAME, None).await.unwrap());
        for i in 0..100 {
            let repo = format!("reopsitory_{i}");
            assert!(db
                .repository_exists(&repo, GITEA_USERNAME, hostname.as_str())
                .await
                .unwrap())
        }
        assert!(db.forge_exists(&hostname).await.unwrap());
    }
}
