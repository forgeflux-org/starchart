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
use actix_web::web;
use actix_web::{HttpResponse, Responder};
use actix_web_codegen_const_routes::get;

pub use api_routes::*;

use crate::pages::chart::home::{OptionalPage, Page};
use crate::search;
use crate::WebFederate;
use crate::{errors::*, WebDB};

const LIMIT: u32 = 50;

#[get(path = "ROUTES.forges")]
pub async fn forges(db: WebDB, q: web::Query<OptionalPage>) -> ServiceResult<impl Responder> {
    let q = q.into_inner();
    let q: Page = q.into();
    let offset = q.page * LIMIT;
    let forges = db.get_all_forges(offset, LIMIT).await?;

    Ok(HttpResponse::Ok().json(forges))
}

#[get(path = "ROUTES.get_latest")]
pub async fn lastest(federate: WebFederate) -> ServiceResult<impl Responder> {
    let latest = federate.latest_tar_json().await.unwrap();
    Ok(HttpResponse::Ok().json(latest))
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(lastest);
    cfg.service(forges);
    search::services(cfg);
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
    async fn list_forges_works() {
        const URL: &str = "https://list-forges-works-test.example.com";
        const HTML_PROFILE_URL: &str = "https://list-forges-works-test.example.com/user1";
        const USERNAME: &str = "user1";

        const REPO_NAME: &str = "asdlkfjaldsfjaksdf";
        const HTML_REPO_URL: &str =
            "https://list-forges-works-test.example.com/user1/asdlkfjaldsfjaksdf";
        const TAGS: [&str; 3] = ["test", "starchart", "spider"];

        let (db, ctx, federate, _tmpdir) = sqlx_sqlite::get_ctx().await;
        let app = get_app!(ctx, db, federate).await;

        let url = Url::parse(URL).unwrap();

        let create_forge_msg = CreateForge {
            url: url.clone(),
            forge_type: ForgeImplementation::Gitea,
            import: false,
        };

        let _ = db.delete_forge_instance(&create_forge_msg.url).await;
        db.create_forge_instance(&create_forge_msg).await.unwrap();
        assert!(
            db.forge_exists(&create_forge_msg.url).await.unwrap(),
            "forge creation failed, forge existence check failure"
        );

        // test starts
        let lisit_res_resp = get_request!(&app, ROUTES.forges);
        assert_eq!(lisit_res_resp.status(), StatusCode::OK);
        let forges_list: Vec<Forge> = test::read_body_json(lisit_res_resp).await;
        assert!(!forges_list.is_empty());
        assert!(forges_list
            .iter()
            .any(|f| f.url == create_forge_msg.url.to_string()));
    }
}
