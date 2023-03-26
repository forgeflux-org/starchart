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
use actix_web::http::header::ContentType;
use actix_web::{HttpResponse, Responder};
use actix_web_codegen_const_routes::post;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use tera::Context;

use db_core::prelude::*;

use crate::errors::ServiceResult;
use crate::pages::errors::*;
use crate::settings::Settings;
use crate::*;

pub use crate::pages::*;

pub const TITLE: &str = "Search";
pub const SEARCH_QUERY_KEY: &str = "search_query";
pub const SEARCH_RESULTS: TemplateFile =
    TemplateFile::new("search_results", "pages/chart/search.html");

pub struct SearchPage {
    ctx: RefCell<Context>,
}

impl CtxError for SearchPage {
    fn with_error(&self, e: &ReadableError) -> String {
        self.ctx.borrow_mut().insert(ERROR_KEY, e);
        self.render()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default, Deserialize, Serialize)]
pub struct SearchPagePayload {
    pub repos: Vec<Repository>,
}

impl SearchPage {
    fn new(settings: &Settings, payload: &SearchPagePayload, search_query: Option<&str>) -> Self {
        let ctx = RefCell::new(ctx(settings));
        ctx.borrow_mut().insert(TITLE_KEY, TITLE);
        ctx.borrow_mut().insert(PAYLOAD_KEY, payload);
        if let Some(search_query) = search_query {
            ctx.borrow_mut().insert(SEARCH_QUERY_KEY, search_query);
        }
        Self { ctx }
    }

    pub fn render(&self) -> String {
        TEMPLATES
            .render(SEARCH_RESULTS.name, &self.ctx.borrow())
            .unwrap()
    }

    pub fn page(s: &Settings, payload: &SearchPagePayload, search_query: Option<&str>) -> String {
        let p = Self::new(s, payload, search_query);
        p.render()
    }
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(search);
}

#[post(path = "PAGES.search")]
pub async fn search(
    payload: web::Form<crate::search::SearchRepositoryReq>,
    ctx: WebCtx,
    db: WebDB,
) -> PageResult<impl Responder, SearchPage> {
    async fn _search(
        ctx: &ArcCtx,
        db: &BoxDB,
        query: String,
    ) -> ServiceResult<Vec<db_core::Repository>> {
        let responses = ctx.search_repository(db, query).await?;

        Ok(responses)
    }

    let query = payload.into_inner().query;
    let repos = _search(&ctx, &db, query.clone()).await.map_err(|e| {
        let x = SearchPagePayload::default();
        PageError::new(SearchPage::new(&ctx.settings, &x, Some(&query)), e)
    })?;

    let payload = SearchPagePayload { repos };
    let page = SearchPage::page(&ctx.settings, &payload, Some(&query));

    let html = ContentType::html();
    Ok(HttpResponse::Ok().content_type(html).body(page))
}
