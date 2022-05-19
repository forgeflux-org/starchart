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
use actix_web::http::{self, header::ContentType};
use actix_web::{HttpResponse, Responder};
use actix_web_codegen_const_routes::{get, post};
use log::info;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use tera::Context;
use url::Url;

use db_core::prelude::*;

use crate::errors::ServiceResult;
use crate::pages::errors::*;
use crate::settings::Settings;
use crate::verify::TXTChallenge;
use crate::*;

pub use crate::pages::*;

pub const TITLE: &str = "Setup spidering";
pub const AUTH_ADD: TemplateFile = TemplateFile::new("auth_add", "pages/auth/add.html");
pub const AUTH_CHALLENGE: TemplateFile =
    TemplateFile::new("auth_challenge", "pages/auth/challenge.html");

pub struct VerifyChallenge {
    ctx: RefCell<Context>,
}

impl CtxError for VerifyChallenge {
    fn with_error(&self, e: &ReadableError) -> String {
        self.ctx.borrow_mut().insert(ERROR_KEY, e);
        self.render()
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct VerifyChallengePayload {
    pub hostname: String,
}

impl VerifyChallenge {
    fn new(settings: &Settings, payload: &Challenge) -> Self {
        let ctx = RefCell::new(ctx(settings));
        ctx.borrow_mut().insert(TITLE_KEY, TITLE);
        ctx.borrow_mut().insert(PAYLOAD_KEY, payload);
        ctx.borrow_mut()
            .insert("form_url", &PAGES.auth.verify_get(&payload.key));
        Self { ctx }
    }

    pub fn render(&self) -> String {
        TEMPLATES.render(AUTH_ADD.name, &self.ctx.borrow()).unwrap()
    }

    pub fn page(s: &Settings, payload: &Challenge) -> String {
        let p = Self::new(s, payload);
        p.render()
    }
}

#[get(path = "PAGES.auth.verify")]
pub async fn get_add(
    ctx: WebCtx,
    db: WebDB,
    query: web::Query<VerifyChallengePayload>,
) -> PageResult<impl Responder, VerifyChallenge> {
    let payload = query.into_inner();
    let value = _get_challenge(&payload, &ctx, &db).await.map_err(|e| {
        let challenge = Challenge {
            key: payload.hostname,
            value: "".into(),
            hostname: "".into(),
        };

        PageError::new(VerifyChallenge::new(&ctx.settings, &challenge), e)
    })?;

    let login = VerifyChallenge::page(&ctx.settings, &value);
    let html = ContentType::html();
    Ok(HttpResponse::Ok().content_type(html).body(login))
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(get_add);
    cfg.service(add_submit);
}

async fn _get_challenge(
    payload: &VerifyChallengePayload,
    ctx: &ArcCtx,
    db: &BoxDB,
) -> ServiceResult<Challenge> {
    let value = db.get_dns_challenge(&payload.hostname).await?;
    Ok(value)
}

#[post(path = "PAGES.auth.verify")]
pub async fn add_submit(
    payload: web::Form<VerifyChallengePayload>,
    ctx: WebCtx,
    db: WebDB,
    federate: WebFederate,
) -> PageResult<impl Responder, VerifyChallenge> {
    let payload = payload.into_inner();
    let value = _get_challenge(&payload, &ctx, &db).await.map_err(|e| {
        let challenge = Challenge {
            key: payload.hostname.clone(),
            value: "".into(),
            hostname: "".into(),
        };

        PageError::new(VerifyChallenge::new(&ctx.settings, &challenge), e)
    })?;

    let challenge = TXTChallenge {
        key: payload.hostname,
        value: value.value,
    };

    match challenge.verify_txt().await {
        Ok(true) => {
            let _ = db.delete_dns_challenge(&challenge.key).await;

            let ctx = ctx.clone();
            let federate = federate.clone();
            let db = db.clone();
            let fut = async move {
                ctx.crawl(&value.hostname, &db, &federate).await;
            };

            tokio::spawn(fut);
            Ok(HttpResponse::Found()
                .insert_header((http::header::LOCATION, PAGES.home))
                .finish())
        }
        _ => Ok(HttpResponse::Found()
            .insert_header((
                http::header::LOCATION,
                PAGES.auth.verify_get(&challenge.key),
            ))
            .finish()),
    }
}

//#[cfg(test)]
//mod tests {
//    use actix_web::http::StatusCode;
//    use actix_web::test;
//    use url::Url;
//
//    use super::VerifyChallenge;
//    use super::VerifyChallengePayload;
//    use super::TXTChallenge;
//    use crate::errors::*;
//    use crate::pages::errors::*;
//    use crate::settings::Settings;
//
//    use db_core::prelude::*;
//
//    #[cfg(test)]
//    mod isolated {
//        use crate::errors::ServiceError;
//        use crate::pages::auth::add::{VerifyChallenge, VerifyChallengePayload, ReadableError};
//        use crate::pages::errors::*;
//        use crate::settings::Settings;
//
//        #[test]
//        fn add_page_works() {
//            let settings = Settings::new().unwrap();
//            VerifyChallenge::page(&settings);
//            let payload = VerifyChallengePayload {
//                hostname: "https://example.com".into(),
//            };
//            let page = VerifyChallenge::new(&settings, Some(&payload));
//            page.with_error(&ReadableError::new(&ServiceError::ClosedForRegistration));
//            page.render();
//        }
//    }
//
//    #[actix_rt::test]
//    async fn add_routes_work() {
//        use crate::tests::*;
//        use crate::*;
//        const BASE_DOMAIN: &str = "add_routes_work.example.org";
//
//        let (db, ctx, federate, _tmpdir) = sqlx_sqlite::get_ctx().await;
//        let app = get_app!(ctx, db, federate).await;
//
//        let payload = VerifyChallengePayload {
//            hostname: format!("https://{BASE_DOMAIN}"),
//        };
//
//        println!("{}", payload.hostname);
//
//        let hostname = get_hostname(&Url::parse(&payload.hostname).unwrap());
//        let key = TXTChallenge::get_challenge_txt_key(&ctx, &hostname);
//
//        db.delete_dns_challenge(&key).await.unwrap();
//        assert!(!db.dns_challenge_exists(&key).await.unwrap());
//
//        let resp = test::call_service(
//            &app,
//            post_request!(&payload, PAGES.auth.add, FORM).to_request(),
//        )
//        .await;
//        if resp.status() != StatusCode::FOUND {
//            let resp_err: ErrorToResponse = test::read_body_json(resp).await;
//            panic!("{}", resp_err.error);
//        }
//        assert_eq!(resp.status(), StatusCode::FOUND);
//
//        assert!(db.dns_challenge_exists(&key).await.unwrap());
//
//        let challenge = db.get_dns_challenge_solution(&key).await.unwrap();
//
//        // replay config
//        let resp = test::call_service(
//            &app,
//            post_request!(&payload, PAGES.auth.add, FORM).to_request(),
//        )
//        .await;
//
//        assert_eq!(resp.status(), StatusCode::FOUND);
//
//        assert!(db.dns_challenge_exists(&key).await.unwrap());
//        assert_eq!(
//            challenge,
//            db.get_dns_challenge_solution(&key).await.unwrap()
//        );
//    }
//}
