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

use serde::Serialize;

use crate::data::Data;
pub use crate::db::BoxDB;
use crate::settings::{DBType, Settings};

//pub mod sqlx_postgres {
//    use super::*;
//
//    pub async fn get_data() -> (BoxDB, Arc<Data>) {
//        let url = env::var("POSTGRES_DATABASE_URL").unwrap();
//        let mut settings = Settings::new().unwrap();
//        settings.database.url = url.clone();
//        settings.database.database_type = DBType::Postgres;
//        let db = pg::get_data(Some(settings.clone())).await;
//        (db, Data::new(Some(settings)))
//    }
//}

pub mod sqlx_sqlite {
    use super::*;
    use crate::db::sqlite;

    pub async fn get_data() -> (BoxDB, Arc<Data>) {
        let url = env::var("SQLITE_DATABASE_URL").unwrap();
        println!("found db url: {url}");
        let mut settings = Settings::new().unwrap();
        settings.database.url = url.clone();
        settings.database.database_type = DBType::Sqlite;
        let db = sqlite::get_data(Some(settings.clone())).await;
        (db, Data::new(settings).await)
    }
}
