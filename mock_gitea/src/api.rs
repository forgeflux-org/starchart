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
use actix_web::web;
use actix_web::{HttpResponse, Responder};
use actix_web_codegen_const_routes::get;
use serde::{Deserialize, Serialize};

use crate::errors::*;
use crate::*;

pub const ROUTES: Api = Api::new();

#[derive(Deserialize, Serialize, Clone, Debug, Eq, PartialEq)]
pub struct Api {
    pub repos_search: &'static str,
    pub repo_tags: &'static str,
}

impl Api {
    const fn new() -> Api {
        let repos_search = "/api/v1/repos/search";
        let repo_tags = "/api/v1/{username}/{name}/topics";
        Api {
            repos_search,
            repo_tags,
        }
    }

    pub fn get_tags_url(&self, username: &str, name: &str) -> String {
        self.repo_tags
            .replace("{username}", username)
            .replace("{name}", name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionalPage {
    pub page: u32,
    pub limit: u32,
}

#[get(path = "ROUTES.repos_search")]
pub async fn search(db: WebDB, q: web::Query<OptionalPage>) -> ServiceResult<impl Responder> {
    let offset = q.page * q.limit;
    let repos = db.get_repositories(offset, q.limit).await?;
    Ok(HttpResponse::Ok().json(repos))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryPath {
    pub username: String,
    pub name: String,
}

#[get(path = "ROUTES.repo_tags")]
pub async fn tags(db: WebDB, p: web::Path<RepositoryPath>) -> ServiceResult<impl Responder> {
    let tags = db.get_tags(&p.username, &p.name).await?;
    Ok(HttpResponse::Ok().json(tags))
}

pub fn services(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(search);
    cfg.service(tags);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::*;

    use actix_web::http::StatusCode;
    use actix_web::test;

    #[actix_rt::test]
    async fn search_works_test() {
        use crate::*;

        const USERNAME: &str = "db_works";
        const REPO_NAMES: &str = "search_works_repo_";
        const TAGS: [&str; 4] = [
            "search_works_tag_1",
            "search_works_tag_2",
            "search_works_tag_3",
            "search_works_tag_4",
        ];

        let (db, ctx) = sqlx_sqlite::get_ctx().await;
        let app = get_app!(ctx, db.clone()).await;

        let repos = 100;

        for count in 0..100 {
            let msg = crate::db::AddRepository {
                name: &format!("{REPO_NAMES}{count}"),
                username: USERNAME,
                tags: &TAGS,
            };

            db.add_repository(msg).await.unwrap();
        }

        let mut page = 0;
        let limit = 10;

        while page <= repos / limit {
            let url = format!("{}?page={page}&limit={limit}", ROUTES.repos_search);

            let repos_resp = get_request!(&app, &url);
            assert_eq!(repos_resp.status(), StatusCode::OK);
            let repos: Vec<crate::db::Repository> = test::read_body_json(repos_resp).await;
            assert!(!repos.is_empty());
            page += 1;

            for r in repos.iter() {
                let tags_resp = get_request!(&app, &ROUTES.get_tags_url(&r.username, &r.name));
                assert_eq!(tags_resp.status(), StatusCode::OK);
                let t: Vec<String> = test::read_body_json(tags_resp).await;
                assert!(!t.is_empty());
            }
        }
        assert_eq!(page, 11);
    }
}
