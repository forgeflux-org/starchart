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
use serde::{Deserialize, Serialize};

use db_core::Repository;

pub const ROUTES: Api = Api::new();

#[derive(Deserialize, Serialize, Clone, Debug, Eq, PartialEq)]
pub struct Search {
    pub repository: &'static str,
}

impl Search {
    const fn new() -> Search {
        let repository = "/api/v1/search/repository";
        Search { repository }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, Eq, PartialEq)]
pub struct Api {
    pub get_latest: &'static str,
    pub forges: &'static str,
    pub search: Search,
}

impl Api {
    const fn new() -> Api {
        let get_latest = "/api/v1/federated/latest";
        let forges = "/api/v1/forges/list";
        let search = Search::new();
        Api {
            get_latest,
            search,
            forges,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, Eq, PartialEq)]
pub struct LatestResp {
    pub latest: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct SearchRepositoryReq {
    pub query: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct SearchRepositoryResp {
    pub repositories: Vec<Repository>,
}