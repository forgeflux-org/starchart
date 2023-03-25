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

use gitea::schema::{self, *};

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
        let repo_tags = "/api/v1/repos/{username}/{name}/topics";
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
pub async fn search(
    db: WebDB,
    ctx: WebCtx,
    q: web::Query<OptionalPage>,
) -> ServiceResult<impl Responder> {
    let offset = q.page * q.limit;
    let mut repos: Vec<db::Repository> = db.get_repositories(offset, q.limit).await?;
    let repos: Vec<schema::Repository> = repos
        .drain(0..)
        .map(|r| {
            let mut repo = schema::Repository::default();
            repo.full_name = format!("{}/{}", r.username, r.name);
            repo.html_url = format!(
                "http://{}:{}/{}/{}",
                ctx.settings.server.domain, ctx.settings.server.port, r.username, r.name
            );
            repo.name = r.name;
            repo.owner.id = r.user_id as usize;
            repo.owner.username = r.username.clone();
            repo.owner.login = r.username.clone();
            repo.owner.full_name = r.username;
            repo
        })
        .collect();
    let resp = SearchResults {
        ok: true,
        data: repos,
    };
    Ok(HttpResponse::Ok().json(resp))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryPath {
    pub username: String,
    pub name: String,
}

#[get(path = "ROUTES.repo_tags")]
pub async fn tags(db: WebDB, p: web::Path<RepositoryPath>) -> ServiceResult<impl Responder> {
    let topics = db.get_tags(&p.username, &p.name).await?;
    let res = schema::Topics { topics };
    Ok(HttpResponse::Ok().json(res))
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
            let repos: schema::SearchResults = test::read_body_json(repos_resp).await;
            assert!(!repos.data.is_empty());
            page += 1;

            for r in repos.data.iter() {
                let tags_resp =
                    get_request!(&app, &ROUTES.get_tags_url(&r.owner.username, &r.name));
                assert_eq!(tags_resp.status(), StatusCode::OK);
                let t: schema::Topics = test::read_body_json(tags_resp).await;
                assert!(!t.topics.is_empty());
            }
        }
        assert_eq!(page, 11);
    }
}
