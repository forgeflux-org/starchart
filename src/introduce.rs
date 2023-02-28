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

use actix_web::web;
use actix_web::{HttpResponse, Responder};
use actix_web_codegen_const_routes::get;
use actix_web_codegen_const_routes::post;
use url::Url;

pub use api_routes::*;
use db_core::prelude::*;

use crate::ctx::Ctx;
use crate::pages::chart::home::{OptionalPage, Page};
use crate::{errors::*, WebDB};

const LIMIT: u32 = 50;

impl Ctx {
    pub async fn import_forges(
        &self,
        starchart_url: Url,
        db: &Box<dyn SCDatabase>,
    ) -> ServiceResult<()> {
        let mut page = 1;
        loop {
            let clean_starchart_url = clean_url(&starchart_url);
            let mut url = starchart_url.clone();
            url.set_path(ROUTES.forges);
            url.set_query(Some(&format!("page={page}")));
            let mut forges: Vec<Forge> = self
                .client
                .get(url)
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
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
    pub async fn bootstrap(&self, db: &Box<dyn SCDatabase>) -> ServiceResult<()> {
        let mut known_starcharts = HashSet::with_capacity(self.settings.introducer.nodes.len());
        for starchart in self.settings.introducer.nodes.iter() {
            let mut page = 1;
            loop {
                let mut url = starchart.clone();
                url.set_path(ROUTES.introducer.list);
                url.set_query(Some(&format!("page={page}")));
                let mut nodes: Vec<Starchart> = self
                    .client
                    .get(url)
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();

                let mut introduce_url = starchart.clone();
                introduce_url.set_path(ROUTES.introducer.introduce);
                let introduction_payload = Starchart {
                    instance_url: self.settings.introducer.public_url.to_string(),
                };
                self.client
                    .post(introduce_url)
                    .json(&introduction_payload)
                    .send()
                    .await
                    .unwrap();
                if nodes.is_empty() {
                    break;
                }
                for node in nodes.drain(0..) {
                    let node_url = Url::parse(&node.instance_url)?;
                    db.add_starchart_to_introducer(&node_url).await?;
                    if known_starcharts.get(&node_url).is_none() {
                        known_starcharts.insert(node_url.clone());
                    }
                    self.import_forges(node_url, db).await?;
                }
                page += 1;
            }
        }
        Ok(())
    }
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
