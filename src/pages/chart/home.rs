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

pub const TITLE: &str = "Explore";
pub const HOME: TemplateFile = TemplateFile::new("home_page", "pages/chart/index.html");
pub const REPO_INFO: TemplateFile =
    TemplateFile::new("repo_info", "pages/chart/components/repo_info.html");

pub struct HomePage {
    ctx: RefCell<Context>,
}

impl CtxError for HomePage {
    fn with_error(&self, e: &ReadableError) -> String {
        self.ctx.borrow_mut().insert(ERROR_KEY, e);
        self.render()
    }
}

#[derive(Clone, Debug, PartialEq, Default, Deserialize, Serialize)]
pub struct HomePagePayload {
    pub repos: Vec<Repository>,
    pub next_page: String,
    pub prev_page: String,
}

impl HomePage {
    fn new(settings: &Settings, payload: &HomePagePayload) -> Self {
        let ctx = RefCell::new(ctx(settings));
        ctx.borrow_mut().insert(TITLE_KEY, TITLE);
        ctx.borrow_mut().insert(PAYLOAD_KEY, payload);
        Self { ctx }
    }

    pub fn render(&self) -> String {
        TEMPLATES.render(HOME.name, &self.ctx.borrow()).unwrap()
    }

    pub fn page(s: &Settings, payload: &HomePagePayload) -> String {
        let p = Self::new(s, payload);
        p.render()
    }
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(home);
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Page {
    pub page: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptionalPage {
    pub page: Option<u32>,
}

impl From<OptionalPage> for Page {
    fn from(o: OptionalPage) -> Self {
        match o.page {
            Some(page) => Self { page },
            None => Page { page: 2 },
        }
    }
}

#[get(path = "PAGES.home")]
pub async fn home(
    q: web::Query<OptionalPage>,
    ctx: WebCtx,
    db: WebDB,
) -> PageResult<impl Responder, HomePage> {
    let q = q.into_inner();
    async fn _home(ctx: &ArcCtx, db: &BoxDB, p: &Page) -> ServiceResult<Vec<db_core::Repository>> {
        const LIMIT: u32 = 10;
        let responses = db.get_all_repositories(p.page, LIMIT).await?;
        Ok(responses)
    }
    let q: Page = q.into();

    let repos = _home(&ctx, &db, &q).await.map_err(|e| {
        let x = HomePagePayload::default();
        PageError::new(HomePage::new(&ctx.settings, &x), e)
    })?;

    let prev = if q.page == 2 { 1 } else { q.page - 1 };

    let payload = HomePagePayload {
        repos,
        next_page: PAGES.home_next(q.page + 1),
        prev_page: PAGES.home_next(prev),
    };
    let page = HomePage::page(&ctx.settings, &payload);

    let html = ContentType::html();
    Ok(HttpResponse::Ok().content_type(html).body(page))
}
