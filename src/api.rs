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
use serde::{Deserialize, Serialize};

use crate::errors::*;
use crate::WebFederate;

pub const ROUTES: Api = Api::new();

#[derive(Deserialize, Serialize, Clone, Debug, Eq, PartialEq)]
pub struct Api {
    pub get_latest: &'static str,
}

impl Api {
    const fn new() -> Api {
        let get_latest = "/api/v1/federated/latest";
        Api { get_latest }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, Eq, PartialEq)]
pub struct LatestResp {
    pub latest: String,
}

impl LatestResp {
    pub async fn new(federate: &WebFederate) -> Self {
        let latest = federate.latest_tar().await.unwrap();
        LatestResp { latest }
    }
}

#[get(path = "ROUTES.get_latest")]
pub async fn lastest(federate: WebFederate) -> ServiceResult<impl Responder> {
    let latest = LatestResp::new(&federate).await;
    Ok(HttpResponse::Ok().json(latest))
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(lastest);
}
