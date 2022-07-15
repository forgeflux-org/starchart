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
//! Test utilities
use crate::prelude::*;

/// adding forge works
pub async fn adding_forge_works<'a, T: SCDatabase>(
    db: &T,
    create_forge_msg: CreateForge,
    add_user_msg: AddUser<'a>,
    add_user_msg2: AddUser<'a>,
    add_repo_msg: AddRepository<'a>,
) {
    let _ = db.delete_forge_instance(&create_forge_msg.url).await;
    db.create_forge_instance(&create_forge_msg).await.unwrap();
    assert!(
        db.forge_exists(&create_forge_msg.url).await.unwrap(),
        "forge creation failed, forge existence check failure"
    );

    {
        let forge = db.get_forge(&create_forge_msg.url).await.unwrap();
        let forges = db.get_all_forges(0, 10).await.unwrap();
        assert_eq!(forges.len(), 1);

        assert_eq!(
            forges.get(0).as_ref().unwrap().forge_type,
            create_forge_msg.forge_type
        );
        assert_eq!(
            forges.get(0).as_ref().unwrap().url,
            crate::clean_url(&create_forge_msg.url)
        );

        assert_eq!(forge.url, crate::clean_url(&create_forge_msg.url));
        assert_eq!(forge.forge_type, create_forge_msg.forge_type);
    }

    // add user
    db.add_user(&add_user_msg).await.unwrap();
    db.add_user(&add_user_msg2).await.unwrap();
    {
        let db_user = db
            .get_user(add_user_msg.username, &add_user_msg.url)
            .await
            .unwrap();
        assert_eq!(db_user.url, crate::clean_url(&add_user_msg.url));
        assert_eq!(db_user.username, add_user_msg.username);
        assert_eq!(db_user.html_link, add_user_msg.html_link);
        assert_eq!(
            db_user.profile_photo,
            add_user_msg.profile_photo.map(|s| s.to_owned())
        );
    }
    // verify user exists
    assert!(db.user_exists(add_user_msg.username, None).await.unwrap());
    assert!(db
        .user_exists(add_user_msg.username, Some(&add_user_msg.url))
        .await
        .unwrap());

    // add repository
    db.create_repository(&add_repo_msg).await.unwrap();
    // verify repo exists
    assert!(db
        .repository_exists(add_repo_msg.name, add_repo_msg.owner, &add_repo_msg.url)
        .await
        .unwrap());
    // delete repository
    db.delete_repository(add_repo_msg.owner, add_repo_msg.name, &add_repo_msg.url)
        .await
        .unwrap();
    assert!(!db
        .repository_exists(add_repo_msg.name, add_repo_msg.owner, &add_repo_msg.url)
        .await
        .unwrap());

    // delete user
    db.delete_user(add_user_msg.username, &add_user_msg.url)
        .await
        .unwrap();
    assert!(!db
        .user_exists(add_user_msg.username, Some(&add_user_msg.url))
        .await
        .unwrap());
}

/// test if all forge type implementations are loaded into DB
pub async fn forge_type_exists_helper<T: SCDatabase>(db: &T) {
    //for f in [ForgeImplementation::Gitea].iter() {
    //let f = For
    let f = ForgeImplementation::Gitea;
    println!("Testing forge implementation exists for: {}", f.to_str());
    assert!(db.forge_type_exists(&f).await.unwrap());
}
