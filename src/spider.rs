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
use std::future::Future;
use std::sync::Arc;
use std::sync::RwLock;

use log::info;
use tokio::sync::oneshot::{error::TryRecvError, Receiver};
use url::Url;

use db_core::prelude::*;
use forge_core::prelude::*;
use gitea::Gitea;

use crate::ctx::Ctx;
use crate::db::BoxDB;
use crate::federate::ArcFederate;
use crate::ArcCtx;

impl Ctx {
    pub async fn crawl(&self, instance_url: &Url, db: &BoxDB, federate: &ArcFederate) {
        info!("[crawl][{instance_url}] Init crawling");
        let forge: Box<dyn SCForge> =
            Box::new(Gitea::new(instance_url.clone(), self.client.clone()));
        if !forge.is_forge().await {
            unimplemented!("Forge type unimplemented");
        }

        let mut page = 1;
        let url = forge.get_url();
        if !db.forge_exists(url).await.unwrap() {
            info!("[crawl][{url}] Creating forge");
            let msg = CreateForge {
                url: url.clone(),
                forge_type: forge.forge_type(),
                starchart_url: None,
            };

            db.create_forge_instance(&msg).await.unwrap();
        } else if !federate.forge_exists(url).await.unwrap() {
            let forge = db.get_forge(url).await.unwrap();
            let msg = CreateForge {
                url: url.clone(),
                forge_type: forge.forge_type,
                starchart_url: None,
            };
            federate.create_forge_instance(&msg).await.unwrap();
        }

        loop {
            info!("[crawl][{url}] Crawling. page: {page}");
            let res = forge
                .crawl(
                    self.settings.crawler.items_per_api_call,
                    page,
                    self.settings.crawler.wait_before_next_api_call,
                )
                .await;
            if res.repos.is_empty() {
                info!("[crawl][{url}] Finished crawling. pages: {}", page - 1);
                break;
            }

            for (username, u) in res.users.iter() {
                if !db
                    .user_exists(username, Some(forge.get_url()))
                    .await
                    .unwrap()
                {
                    info!("[crawl][{url}] Creating user: {username}");
                    let msg = u.as_ref().into();
                    db.add_user(&msg).await.unwrap();
                    federate.create_user(&msg).await.unwrap();
                }
                if !federate
                    .user_exists(username, forge.get_url())
                    .await
                    .unwrap()
                {
                    let msg = u.as_ref().into();
                    federate.create_user(&msg).await.unwrap();
                }
            }

            for r in res.repos.iter() {
                if !db
                    .repository_exists(&r.name, &r.owner.username, &r.url)
                    .await
                    .unwrap()
                {
                    info!("[crawl][{url}] Creating repository: {}", r.name);
                    let msg = r.into();
                    db.create_repository(&msg).await.unwrap();
                    federate.create_repository(&msg).await.unwrap();
                }
                if !federate
                    .repository_exists(&r.name, &r.owner.username, &r.url)
                    .await
                    .unwrap()
                {
                    let msg = r.into();
                    federate.create_repository(&msg).await.unwrap();
                }
            }

            page += 1;
        }
    }
}

pub struct Crawler {
    rx: RwLock<Option<Receiver<bool>>>,
    ctx: ArcCtx,
    db: BoxDB,
    federate: ArcFederate,
}

impl Crawler {
    pub fn new(rx: Receiver<bool>, ctx: ArcCtx, db: BoxDB, federate: ArcFederate) -> Arc<Self> {
        let rx = RwLock::new(Some(rx));
        Arc::new(Self {
            rx,
            ctx,
            db,
            federate,
        })
    }

    pub fn is_running(&self) -> bool {
        self.rx.read().unwrap().is_some()
    }

    fn shutdown(&self) -> bool {
        let res = if let Some(rx) = self.rx.write().unwrap().as_mut() {
            //            let rx = self.rx.as_mut().unwrap();
            match rx.try_recv() {
                // The channel is currently empty
                Ok(x) => {
                    info!("Received signal from tx");
                    x
                }
                Err(TryRecvError::Empty) => false,
                Err(TryRecvError::Closed) => {
                    info!("Closed");
                    true
                }
            }
        } else {
            true
        };
        if res {
            let mut rx = self.rx.write().unwrap();
            *rx = None;
        }
        res
    }

    // static is justified since the crawler will be initialized when the program starts
    // and only shutdown when the program exits
    pub async fn start(c: Arc<Crawler>) -> impl Future {
        if c.shutdown() {
            info!("Stopping crawling job");
            return tokio::spawn(tokio::time::sleep(std::time::Duration::new(0, 0)));
        }

        let fut = async move {
            const LIMIT: u32 = 2;
            let mut page = 0;
            loop {
                info!("Running crawling job");
                let offset = page * LIMIT;
                if c.shutdown() {
                    break;
                }

                let forges = c.db.get_all_forges(false, offset, LIMIT).await.unwrap();
                if forges.is_empty() {
                    c.federate.tar().await.unwrap();
                    page = 0;
                    tokio::time::sleep(std::time::Duration::new(c.ctx.settings.crawler.ttl, 0))
                        .await;
                    if c.shutdown() {
                        info!("Stopping crawling job");
                        break;
                    }

                    continue;
                }
                for forge in forges.iter() {
                    if c.shutdown() {
                        info!("Stopping crawling job");
                        break;
                    }
                    c.ctx
                        .crawl(&Url::parse(&forge.url).unwrap(), &c.db, &c.federate)
                        .await;
                    page += 1;
                }

                if c.shutdown() {
                    info!("Stopping crawling job");
                    break;
                }
            }
        };

        tokio::spawn(fut)
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::sqlx_sqlite;

    use url::Url;

    pub const GITEA_HOST: &str = "http://localhost:8080";
    pub const GITEA_USERNAME: &str = "bot";

    #[actix_rt::test]
    async fn crawl_gitea() {
        let (db, ctx, federate, _tmp_dir) = sqlx_sqlite::get_ctx().await;
        let url = Url::parse(GITEA_HOST).unwrap();
        ctx.crawl(&url, &db, &federate).await;
        //        let hostname = get_hostname(&Url::parse(GITEA_HOST).unwrap());
        assert!(db.forge_exists(&url).await.unwrap());
        assert!(db.user_exists(GITEA_USERNAME, Some(&url)).await.unwrap());
        assert!(db.user_exists(GITEA_USERNAME, None).await.unwrap());
        for i in 0..100 {
            let repo = format!("repository_{i}");
            assert!(db
                .repository_exists(&repo, GITEA_USERNAME, &url)
                .await
                .unwrap())
        }
        assert!(db.forge_exists(&url).await.unwrap());
    }

    //    #[actix_rt::test]
    //    async fn crawlerd() {
    //        use super::*;
    //use tokio::sync::oneshot;
    //
    //        let (db, ctx, federate, _tmp_dir) = sqlx_sqlite::get_ctx().await;
    //    let (kill_crawler, rx) = oneshot::channel();
    //        let crawler = Crawler::new(rx, ctx.clone(), db, federate);
    //        let fut = tokio::spawn(Crawler::start(crawler.clone()));
    //        assert!(crawler.is_running());
    //
    //                    tokio::time::sleep(std::time::Duration::new(2, 0))
    //                        .await;
    //
    //
    //        kill_crawler.send(true).unwrap();
    //                    tokio::time::sleep(std::time::Duration::new(2, 0))
    //                        .await;
    //
    //
    //        fut.await.unwrap().await;
    //        assert!(!crawler.is_running());
    //    }
}
