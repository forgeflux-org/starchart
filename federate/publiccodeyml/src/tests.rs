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
use mktemp::Temp;
use url::Url;

use crate::*;
use federate_core::tests;

#[actix_rt::test]
async fn everything_works() {
    const URL: &str = "https://test-gitea.example.com";
    const HTML_PROFILE_URL: &str = "https://test-gitea.example.com/user1";
    const USERNAME: &str = "user1";

    const REPO_NAME: &str = "starchart";
    const HTML_REPO_URL: &str = "https://test-gitea.example.com/user1/starchart";
    const TAGS: [&str; 3] = ["test", "starchart", "spider"];

    let tmp_dir = Temp::new_dir().unwrap();

    let url = Url::parse(URL).unwrap();

    let create_forge_msg = CreateForge {
        url: url.clone(),
        forge_type: ForgeImplementation::Gitea,
    };

    let add_user_msg = AddUser {
        url: url.clone(),
        html_link: HTML_PROFILE_URL,
        profile_photo: None,
        username: USERNAME,
    };

    let add_repo_msg = AddRepository {
        html_link: HTML_REPO_URL,
        name: REPO_NAME,
        tags: Some(TAGS.into()),
        owner: USERNAME,
        website: None,
        description: None,
        url: url.clone(),
    };

    let pcc = PccFederate::new(tmp_dir.to_str().unwrap().to_string())
        .await
        .unwrap();
    tests::adding_forge_works(&pcc, create_forge_msg, add_user_msg, add_repo_msg).await;
}
