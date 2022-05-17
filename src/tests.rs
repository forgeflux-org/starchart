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

use std::env;
pub use std::sync::Arc;

use crate::ctx::Ctx;
pub use crate::db::BoxDB;
pub use crate::federate::{get_federate, BoxFederate};
use crate::settings::{DBType, Settings};

//pub mod sqlx_postgres {
//    use super::*;
//
//    pub async fn get_ctx() -> (BoxDB, Arc<Ctx>) {
//        let url = env::var("POSTGRES_DATABASE_URL").unwrap();
//        let mut settings = Settings::new().unwrap();
//        settings.database.url = url.clone();
//        settings.database.database_type = DBType::Postgres;
//        let db = pg::get_data(Some(settings.clone())).await;
//        (db, Ctx::new(Some(settings)))
//    }

pub mod sqlx_sqlite {
    use super::*;
    use crate::db::sqlite;
    use mktemp::Temp;

    pub async fn get_ctx() -> (BoxDB, Arc<Ctx>, BoxFederate, Temp) {
        let url = env::var("SQLITE_DATABASE_URL").unwrap();
        env::set_var("DATABASE_URL", &url);
        println!("found db url: {url}");
        let mut settings = Settings::new().unwrap();
        settings.database.url = url.clone();
        settings.database.database_type = DBType::Sqlite;
        let db = sqlite::get_data(Some(settings.clone())).await;

        let tmp_dir = Temp::new_dir().unwrap();
        settings.repository.root = tmp_dir.to_str().unwrap().to_string();
        let federate = get_federate(Some(settings.clone())).await;

        (db, Ctx::new(settings).await, federate, tmp_dir)
    }
}
