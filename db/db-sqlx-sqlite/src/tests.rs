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
use std::env;

use sqlx::sqlite::SqlitePoolOptions;
use url::Url;

use crate::*;

use db_core::tests::*;

#[actix_rt::test]
async fn everything_works() {
    const URL: &str = "https://test-gitea.example.com";
    const HTML_PROFILE_URL: &str = "https://test-gitea.example.com/user1";
    const HTML_PROFILE_PHOTO_URL_2: &str = "https://test-gitea.example.com/profile-photo/user2";
    const USERNAME: &str = "user1";
    const USERNAME2: &str = "user2";

    const REPO_NAME: &str = "starchart";
    const HTML_REPO_URL: &str = "https://test-gitea.example.com/user1/starchart";
    const TAGS: [&str; 3] = ["test", "starchart", "spider"];

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

    let add_user_msg_2 = AddUser {
        url: url.clone(),
        html_link: HTML_PROFILE_PHOTO_URL_2,
        profile_photo: Some(HTML_PROFILE_PHOTO_URL_2),
        username: USERNAME2,
        import: false,
    };

    let db = {
        let url = env::var("SQLITE_DATABASE_URL").expect("Set SQLITE_DATABASE_URL env var");
        let pool_options = SqlitePoolOptions::new().max_connections(2);
        let connection_options = ConnectionOptions::Fresh(Fresh { pool_options, url });
        let db = connection_options.connect().await.unwrap();
        db.migrate().await.unwrap();
        db
    };

    let add_repo_msg = AddRepository {
        html_link: HTML_REPO_URL,
        name: REPO_NAME,
        tags: Some(TAGS.into()),
        owner: USERNAME,
        website: "https://starcahrt-sqlite-test.example.org".into(),
        description: "starchart sqlite test repo sescription".into(),
        url,
        import: false,
    };

    adding_forge_works(
        &db,
        create_forge_msg,
        add_user_msg,
        add_user_msg_2,
        add_repo_msg,
    )
    .await;
}

#[actix_rt::test]
async fn introducer_works() {
    let url = env::var("SQLITE_DATABASE_URL").expect("Set SQLITE_DATABASE_URL env var");
    let pool_options = SqlitePoolOptions::new().max_connections(2);
    let connection_options = ConnectionOptions::Fresh(Fresh { pool_options, url });
    let db = connection_options.connect().await.unwrap();

    let instance_url = Url::parse("https://introducer_works_sqlite_sqlx.example.com").unwrap();
    instance_introducer_helper(&db, &instance_url).await;
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

#[actix_rt::test]
async fn mini_index_test() {
    let url = env::var("SQLITE_DATABASE_URL").expect("Set SQLITE_DATABASE_URL env var");
    let pool_options = SqlitePoolOptions::new().max_connections(2);
    let connection_options = ConnectionOptions::Fresh(Fresh { pool_options, url });
    let db = connection_options.connect().await.unwrap();

    db.migrate().await.unwrap();
    mini_index_helper(&db).await;
}
