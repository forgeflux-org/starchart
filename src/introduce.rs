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
use actix_web::web;
use actix_web::{HttpResponse, Responder};
use actix_web_codegen_const_routes::get;

pub use api_routes::*;

use crate::pages::chart::home::{OptionalPage, Page};
use crate::search;
use crate::WebFederate;
use crate::{errors::*, WebDB};

const LIMIT: u32 = 50;

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

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(list_introductions);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::*;
    use crate::*;

    use actix_web::http::StatusCode;
    use actix_web::test;
    use db_core::prelude::*;
    use url::Url;

    #[actix_rt::test]
    async fn introductions_works() {
        const STARCHART_URL: &str = "https://introductions-works.example.com/";

        let (db, ctx, federate, _tmpdir) = sqlx_sqlite::get_ctx().await;
        let app = get_app!(ctx, db, federate).await;
        db.add_starchart_to_introducer(&Url::parse(STARCHART_URL).unwrap())
            .await
            .unwrap();
        let introductions_resp = get_request!(&app, ROUTES.introducer.list);
        assert_eq!(introductions_resp.status(), StatusCode::OK);
        let introductions: Vec<Starchart> = test::read_body_json(introductions_resp).await;
        assert!(!introductions.is_empty());

        assert!(introductions
            .iter()
            .any(|i| i.instance_url == STARCHART_URL));
    }
}
