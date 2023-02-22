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
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use tera::Context;
use url::Url;

use crate::pages::errors::*;
use crate::settings::Settings;
use crate::*;

pub use crate::pages::*;

pub const TITLE: &str = "Setup spidering";
pub const AUTH_ADD: TemplateFile = TemplateFile::new("auth_add", "pages/auth/add.html");

pub struct AddChallenge {
    ctx: RefCell<Context>,
}

impl CtxError for AddChallenge {
    fn with_error(&self, e: &ReadableError) -> String {
        self.ctx.borrow_mut().insert(ERROR_KEY, e);
        self.render()
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct AddChallengePayload {
    pub hostname: Url,
}

impl AddChallenge {
    fn new(settings: &Settings, payload: Option<&AddChallengePayload>) -> Self {
        let ctx = RefCell::new(ctx(settings));
        ctx.borrow_mut().insert(TITLE_KEY, TITLE);
        if let Some(payload) = payload {
            ctx.borrow_mut().insert(PAYLOAD_KEY, payload);
        }
        Self { ctx }
    }

    pub fn render(&self) -> String {
        TEMPLATES.render(AUTH_ADD.name, &self.ctx.borrow()).unwrap()
    }

    pub fn page(s: &Settings) -> String {
        let p = Self::new(s, None);
        p.render()
    }
}

#[get(path = "PAGES.auth.add")]
pub async fn get_add(ctx: WebCtx) -> impl Responder {
    let login = AddChallenge::page(&ctx.settings);
    let html = ContentType::html();
    HttpResponse::Ok().content_type(html).body(login)
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(get_add);
    cfg.service(add_submit);
}

#[post(path = "PAGES.auth.add")]
pub async fn add_submit(
    payload: web::Form<AddChallengePayload>,
) -> PageResult<impl Responder, AddChallenge> {
    let link = PAGES.auth.verify_get(&payload.hostname.to_string());

    Ok(HttpResponse::Found()
        .insert_header((http::header::LOCATION, link))
        .finish())
}

#[cfg(test)]
mod tests {
    use actix_web::http::StatusCode;
    use actix_web::test;
    use url::Url;

    use super::AddChallengePayload;
    use crate::errors::*;

    #[cfg(test)]
    mod isolated {
        use crate::errors::ServiceError;
        use crate::pages::auth::add::{AddChallenge, AddChallengePayload, ReadableError};
        use crate::pages::errors::*;
        use crate::settings::Settings;

        #[test]
        fn add_page_works() {
            let settings = Settings::new().unwrap();
            AddChallenge::page(&settings);
            let payload = AddChallengePayload {
                hostname: url::Url::parse("https://example.com").unwrap(),
            };
            let page = AddChallenge::new(&settings, Some(&payload));
            page.with_error(&ReadableError::new(&ServiceError::ClosedForRegistration));
            page.render();
        }
    }

    #[actix_rt::test]
    async fn add_routes_work() {
        use crate::tests::*;
        use crate::*;
        const BASE_DOMAIN: &str = "add_routes_work.example.org";

        let (db, ctx, federate, _tmpdir) = sqlx_sqlite::get_ctx().await;
        let app = get_app!(ctx, db, federate).await;

        let payload = AddChallengePayload {
            hostname: Url::parse(&format!("https://{BASE_DOMAIN}")).unwrap(),
        };

        println!("{}", payload.hostname);

        let resp = test::call_service(
            &app,
            post_request!(&payload, PAGES.auth.add, FORM).to_request(),
        )
        .await;
        if resp.status() != StatusCode::FOUND {
            let resp_err: ErrorToResponse = test::read_body_json(resp).await;
            panic!("{}", resp_err.error);
        }
        assert_eq!(resp.status(), StatusCode::FOUND);

        // replay config
        let resp = test::call_service(
            &app,
            post_request!(&payload, PAGES.auth.add, FORM).to_request(),
        )
        .await;

        assert_eq!(resp.status(), StatusCode::FOUND);
    }
}
