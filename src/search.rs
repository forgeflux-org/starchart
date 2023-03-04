use crate::counter::AddSearch;
use crate::master::{AddCounter, GetSite};
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
use crate::{counter, errors::*, WebCtx};
use actix_web::web;
use actix_web::{HttpResponse, Responder};
use actix_web_codegen_const_routes::post;
use db_core::prelude::*;
use url::Url;

use crate::Ctx;
use crate::WebDB;

pub use crate::api::{SearchRepositoryReq, ROUTES};

impl Ctx {
    async fn client_federated_search(
        &self,
        mut starchart_url: Url,
        payload: &SearchRepositoryReq,
    ) -> ServiceResult<Vec<Repository>> {
        starchart_url.set_path(ROUTES.search.repository);
        Ok(self
            .client
            .post(starchart_url)
            .json(&payload)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap())
    }

    pub async fn search_repository(
        &self,
        db: &Box<dyn SCDatabase>,
        query: String,
    ) -> ServiceResult<Vec<Repository>> {
        let query = if query.contains('*') {
            query
        } else {
            format!("*{}*", query)
        };
        let federated_search_payload = SearchRepositoryReq {
            query: query.clone(),
        };
        let local_resp = db.search_repository(&query).await?;
        let mut federated_resp = Vec::default();

        for starchart in db.search_mini_index(&query).await?.iter() {
            if db.is_starchart_imported(&Url::parse(&starchart)?).await? {
                log::debug!("{starchart} is imported");
                continue;
            }
            let addr = if let Some(addr) = self.master.send(GetSite(starchart.clone())).await? {
                addr
            } else {
                self.master
                    .send(AddCounter {
                        id: starchart.clone(),
                        counter: counter::Count {
                            duration: 54,
                            search_threshold: 0,
                        }
                        .into(),
                    })
                    .await?;
                self.master.send(GetSite(starchart.clone())).await?.unwrap()
            };

            let count = addr.send(AddSearch).await?;
            if count > 50 {
                todo!("Clone index");
            } else {
                let resp = self
                    .client_federated_search(Url::parse(starchart)?, &federated_search_payload)
                    .await?;
                federated_resp.extend(resp);
            }
        }

        federated_resp.extend(local_resp);
        Ok(federated_resp)
    }
}

#[post(path = "ROUTES.search.repository")]
pub async fn search_repository(
    payload: web::Json<SearchRepositoryReq>,
    ctx: WebCtx,
    db: WebDB,
) -> ServiceResult<impl Responder> {
    let resp = ctx
        .search_repository(&db, payload.into_inner().query)
        .await?;
    Ok(HttpResponse::Ok().json(resp))
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(search_repository);
}

#[cfg(test)]
mod tests {
    use actix_web::test;
    use url::Url;

    use super::*;
    use actix_web::http::StatusCode;

    use crate::tests::*;
    use crate::*;

    #[actix_rt::test]
    async fn search_works() {
        const URL: &str = "https://search-works-test.example.com";
        const HTML_PROFILE_URL: &str = "https://search-works-test.example.com/user1";
        const USERNAME: &str = "user1";

        const REPO_NAME: &str = "searchsasdf2";
        const HTML_REPO_URL: &str = "https://search-works-test.example.com/user1/searchsasdf2";
        const TAGS: [&str; 3] = ["test", "starchart", "spider"];

        let (db, ctx, federate, _tmpdir) = sqlx_sqlite::get_ctx().await;
        let app = get_app!(ctx, db, federate).await;

        let url = Url::parse(URL).unwrap();

        let create_forge_msg = CreateForge {
            url: url.clone(),
            forge_type: ForgeImplementation::Gitea,
            starchart_url: None,
        };

        let add_user_msg = AddUser {
            url: url.clone(),
            html_link: HTML_PROFILE_URL,
            profile_photo: None,
            username: USERNAME,
            import: false,
        };

        let add_repo_msg = AddRepository {
            html_link: HTML_REPO_URL,
            name: REPO_NAME,
            tags: Some(TAGS.into()),
            owner: USERNAME,
            website: None,
            description: None,
            url,
            import: false,
        };

        let _ = db.delete_forge_instance(&create_forge_msg.url).await;
        db.create_forge_instance(&create_forge_msg).await.unwrap();
        assert!(
            db.forge_exists(&create_forge_msg.url).await.unwrap(),
            "forge creation failed, forge existence check failure"
        );

        // add user
        db.add_user(&add_user_msg).await.unwrap();
        // add repository
        db.create_repository(&add_repo_msg).await.unwrap();
        // verify repo exists
        assert!(db
            .repository_exists(add_repo_msg.name, add_repo_msg.owner, &add_repo_msg.url)
            .await
            .unwrap());

        // test starts

        let payload = SearchRepositoryReq {
            query: REPO_NAME[0..REPO_NAME.len() - 4].to_string(),
        };
        let search_res_resp = test::call_service(
            &app,
            post_request!(&payload, ROUTES.search.repository).to_request(),
        )
        .await;
        assert_eq!(search_res_resp.status(), StatusCode::OK);
        let search_res: Vec<Repository> = test::read_body_json(search_res_resp).await;
        println!("{:?}", search_res);
        assert!(!search_res.is_empty());
        assert_eq!(search_res.first().as_ref().unwrap().name, REPO_NAME);

        let mini_index_resp = get_request!(&app, ROUTES.introducer.get_mini_index);
        assert_eq!(mini_index_resp.status(), StatusCode::OK);
        let mini_index: api_routes::MiniIndex = test::read_body_json(mini_index_resp).await;
        assert!(!mini_index.mini_index.is_empty());
        assert!(mini_index.mini_index.contains(USERNAME));

        // test ends
    }
}
