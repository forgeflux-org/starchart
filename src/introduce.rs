/*
 * ForgeFlux StarChart - A federated software forge spider
 * Copyright (C) 2023  Aravinth Manivannan <realaravinth@batsense.net>
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
use std::collections::HashSet;
use std::future::Future;

use actix_web::web;
use actix_web::{HttpResponse, Responder};
use actix_web_codegen_const_routes::get;
use actix_web_codegen_const_routes::post;
use tokio::sync::oneshot::{self, error::TryRecvError, Sender};
use url::Url;

pub use api_routes::*;
use db_core::prelude::*;

use crate::ctx::Ctx;
use crate::pages::chart::home::{OptionalPage, Page};
use crate::{errors::*, WebCtx, WebDB};

const LIMIT: u32 = 50;

impl Ctx {
    async fn client_get_forges(
        &self,
        starchart_url: &Url,
        page: usize,
    ) -> ServiceResult<Vec<Forge>> {
        if starchart_url == &self.settings.introducer.public_url {
            panic!()
        }
        let mut url = starchart_url.clone();
        url.set_path(ROUTES.forges);
        url.set_query(Some(&format!("page={page}")));
        Ok(self
            .client
            .get(url)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap())
    }
    pub async fn import_forges(
        &self,
        starchart_url: Url,
        db: &Box<dyn SCDatabase>,
    ) -> ServiceResult<()> {
        if starchart_url == self.settings.introducer.public_url {
            panic!()
        }
        let clean_starchart_url = clean_url(&starchart_url);
        let mut page = 1;
        loop {
            let mut forges = self.client_get_forges(&starchart_url, page).await?;

            page += 1;
            if forges.is_empty() {
                break;
            }

            for f in forges.drain(0..) {
                let msg = CreateForge {
                    starchart_url: Some(&clean_starchart_url),
                    url: Url::parse(&f.url).unwrap(),
                    forge_type: f.forge_type,
                };
                db.create_forge_instance(&msg).await?;
            }
        }
        Ok(())
    }

    async fn client_get_introducions(
        &self,
        mut starchart_url: Url,
        page: usize,
    ) -> ServiceResult<Vec<Starchart>> {
        if starchart_url == self.settings.introducer.public_url {
            panic!()
        }
        starchart_url.set_path(ROUTES.introducer.list);
        starchart_url.set_query(Some(&format!("page={page}")));
        Ok(self
            .client
            .get(starchart_url)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap())
    }
    async fn client_get_mini_index(&self, mut starchart_url: Url) -> ServiceResult<MiniIndex> {
        if starchart_url == self.settings.introducer.public_url {
            panic!()
        }
        starchart_url.set_path(ROUTES.introducer.get_mini_index);
        let resp = self.client.get(starchart_url).send().await.unwrap();
        println!("{:?}", resp);
        Ok(resp.json().await.unwrap())
    }

    async fn client_introduce_starchart(&self, mut starchart_url: Url) -> ServiceResult<()> {
        if starchart_url == self.settings.introducer.public_url {
            return Ok(());
        }
        starchart_url.set_path(ROUTES.introducer.introduce);
        let introduction_payload = Starchart {
            instance_url: self.settings.introducer.public_url.to_string(),
        };
        self.client
            .post(starchart_url)
            .json(&introduction_payload)
            .send()
            .await
            .unwrap();

        Ok(())
    }

    pub async fn spawn_bootstrap(
        ctx: WebCtx,
        db: Box<dyn SCDatabase>,
    ) -> ServiceResult<(Sender<bool>, impl Future)> {
        let (tx, mut rx) = oneshot::channel();
        let fut = async move {
            loop {
                let shutdown = match rx.try_recv() {
                    // The channel is currently empty
                    Ok(x) => {
                        log::info!("[introducer] Received signal {x}");
                        x
                    }
                    Err(TryRecvError::Empty) => false,
                    Err(TryRecvError::Closed) => {
                        log::info!("Closed");
                        true
                    }
                };
                if shutdown {
                    break;
                }

                let _ = ctx.bootstrap(&db).await;
                log::info!(
                    "Waiting for {} until re-introducing",
                    ctx.settings.introducer.wait
                );
                tokio::time::sleep(std::time::Duration::new(ctx.settings.introducer.wait, 0)).await;
            }
        };

        let join_handle = tokio::spawn(fut);
        Ok((tx, join_handle))
    }

    pub async fn bootstrap(&self, db: &Box<dyn SCDatabase>) -> ServiceResult<()> {
        async fn run(
            ctx: &Ctx,
            db: &Box<dyn SCDatabase>,
            starchart: &Url,
            known_starcharts: &mut HashSet<Url>,
        ) -> ServiceResult<()> {
            let mut page = 1;
            loop {
                let mut nodes = ctx.client_get_introducions(starchart.clone(), page).await?;

                ctx.client_introduce_starchart(starchart.clone()).await?;

                if nodes.is_empty() {
                    break;
                }

                async fn _bootstrap(
                    ctx: &Ctx,
                    db: &Box<dyn SCDatabase>,
                    known_starcharts: &mut HashSet<Url>,
                    instance_url: &str,
                ) -> ServiceResult<()> {
                    if instance_url == ctx.settings.introducer.public_url.as_str() {
                        return Ok(());
                    }

                    let node_url = Url::parse(instance_url)?;
                    db.add_starchart_to_introducer(&node_url).await?;
                    if known_starcharts.get(&node_url).is_none() {
                        known_starcharts.insert(node_url.clone());
                    }
                    ctx.import_forges(node_url.clone(), db).await?;
                    let mini_index = ctx.client_get_mini_index(node_url.clone()).await?;
                    log::info!(
                        "Received mini_index {} from {node_url}",
                        mini_index.mini_index
                    );
                    db.rm_imported_mini_index(&node_url).await?;
                    db.import_mini_index(&node_url, &mini_index.mini_index)
                        .await?;

                    Ok(())
                }

                _bootstrap(ctx, db, known_starcharts, starchart.as_str()).await?;

                for node in nodes.drain(0..) {
                    if node.instance_url == ctx.settings.introducer.public_url.as_str() {
                        continue;
                    }
                    _bootstrap(ctx, db, known_starcharts, &node.instance_url).await?;
                }
                page += 1;
            }
            Ok(())
        }
        let mut known_starcharts = HashSet::with_capacity(self.settings.introducer.nodes.len());
        for starchart in self.settings.introducer.nodes.iter() {
            run(self, db, starchart, &mut known_starcharts).await?;
        }

        let mut page = 0;
        loop {
            let offset = page * LIMIT;
            let starcharts = db
                .get_all_introduced_starchart_instances(offset, LIMIT)
                .await?;
            for starchart in self.settings.introducer.nodes.iter() {
                run(self, db, starchart, &mut known_starcharts).await?;
            }
            if starcharts.len() < LIMIT as usize {
                break;
            } else {
                page += 1;
            }
        }
        Ok(())
    }
}

#[get(path = "ROUTES.introducer.get_mini_index")]
pub async fn get_mini_index(db: WebDB) -> ServiceResult<impl Responder> {
    let mini_index = db.export_mini_index().await?;

    let resp = MiniIndex { mini_index };

    Ok(HttpResponse::Ok().json(resp))
}

#[get(path = "ROUTES.introducer.list")]
pub async fn list_introductions(
    db: WebDB,
    q: web::Query<OptionalPage>,
) -> ServiceResult<impl Responder> {
    let q = q.into_inner();
    let q: Page = q.into();
    let offset = q.page * LIMIT;
    let starcharts = db
        .get_all_introduced_starchart_instances(offset, LIMIT)
        .await?;

    Ok(HttpResponse::Ok().json(starcharts))
}

#[post(path = "ROUTES.introducer.introduce")]
pub async fn new_introduction(
    db: WebDB,
    payload: web::Json<Starchart>,
) -> ServiceResult<impl Responder> {
    db.add_starchart_to_introducer(&Url::parse(&payload.instance_url)?)
        .await?;
    Ok(HttpResponse::Ok())
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(list_introductions);
    cfg.service(new_introduction);
    cfg.service(get_mini_index);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::*;
    use crate::*;

    use actix_web::http::StatusCode;
    use actix_web::test;
    use url::Url;

    #[actix_rt::test]
    async fn introductions_works() {
        const STARCHART_URL: &str = "https://introductions-works.example.com/";

        let (db, ctx, federate, _tmpdir) = sqlx_sqlite::get_ctx().await;
        let app = get_app!(ctx, db, federate).await;
        db.add_starchart_to_introducer(&Url::parse(STARCHART_URL).unwrap())
            .await
            .unwrap();

        let payload = Starchart {
            instance_url: STARCHART_URL.into(),
        };

        let resp = test::call_service(
            &app,
            post_request!(&payload, ROUTES.introducer.introduce).to_request(),
        )
        .await;
        if resp.status() != StatusCode::OK {
            let resp_err: ErrorToResponse = test::read_body_json(resp).await;
            panic!("{}", resp_err.error);
        }
        assert_eq!(resp.status(), StatusCode::OK);

        let introductions_resp = get_request!(&app, ROUTES.introducer.list);
        assert_eq!(introductions_resp.status(), StatusCode::OK);
        let introductions: Vec<Starchart> = test::read_body_json(introductions_resp).await;
        assert!(!introductions.is_empty());

        assert!(introductions
            .iter()
            .any(|i| i.instance_url == STARCHART_URL));
    }
}
