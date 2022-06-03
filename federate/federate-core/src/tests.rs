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
use crate::*;

/// adding forge works
pub async fn adding_forge_works<'a, T: Federate>(
    ff: &T,
    create_forge_msg: CreateForge<'a>,
    create_user_msg: AddUser<'a>,
    add_repo_msg: AddRepository<'a>,
) {
    let _ = ff.delete_forge_instance(create_forge_msg.hostname).await;
    assert!(!ff.forge_exists(&create_forge_msg.hostname).await.unwrap());
    ff.create_forge_isntance(&create_forge_msg).await.unwrap();
    assert!(ff.forge_exists(&create_forge_msg.hostname).await.unwrap());
    // add user
    ff.create_user(&create_user_msg).await.unwrap();

    // add repository
    ff.create_repository(&add_repo_msg).await.unwrap();

    // tar()
    ff.tar().await.unwrap();

    // delete repository
    ff.delete_repository(add_repo_msg.owner, add_repo_msg.name, add_repo_msg.hostname)
        .await
        .unwrap();

    // delete user
    ff.delete_user(create_user_msg.username, create_user_msg.hostname)
        .await
        .unwrap();

    // delete user
    ff.delete_forge_instance(create_forge_msg.hostname)
        .await
        .unwrap();
}
