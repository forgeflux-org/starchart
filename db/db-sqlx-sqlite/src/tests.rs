/*
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
use sqlx::sqlite::SqlitePoolOptions;
use std::env;

use crate::*;

use db_core::tests::*;

#[actix_rt::test]
async fn everything_works() {
    const HOSTNAME: &str = "test-gitea.example.com";
    let msg = CreateForge {
        hostname: HOSTNAME,
        forge_type: ForgeImplementation::Gitea,
    };
    let url = env::var("SQLITE_DATABASE_URL").expect("Set SQLITE_DATABASE_URL env var");
    let pool_options = SqlitePoolOptions::new().max_connections(2);
    let connection_options = ConnectionOptions::Fresh(Fresh { pool_options, url });
    let db = connection_options.connect().await.unwrap();

    adding_forge_works(&db, msg).await;
}

#[actix_rt::test]
async fn forge_type_exists() {
    let url = env::var("SQLITE_DATABASE_URL").expect("Set SQLITE_DATABASE_URL env var");
    let pool_options = SqlitePoolOptions::new().max_connections(2);
    let connection_options = ConnectionOptions::Fresh(Fresh { pool_options, url });
    let db = connection_options.connect().await.unwrap();

    db.migrate().await.unwrap();
    forge_type_exists_helper(&db).await;
}
