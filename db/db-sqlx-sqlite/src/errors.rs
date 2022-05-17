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
use std::borrow::Cow;

use db_core::dev::*;
use sqlx::Error;

pub fn map_register_err(e: Error) -> DBError {
    if let Error::Database(err) = e {
        if err.code() == Some(Cow::from("2067")) {
            let msg = err.message();
            println!("db err: {msg}");
            if msg.contains("starchart_dns_challenges.hostname") {
                DBError::DuplicateChallengeHostname
            } else if msg.contains("starchart_forges.hostname") {
                DBError::DuplicateHostname
            } else if msg.contains("starchart_dns_challenges.challenge") {
                DBError::DuplicateChallengeText
            } else if msg.contains("starchart_users.html_url") {
                DBError::DuplicateUserLink
            } else if msg.contains("starchart_project_topics.name") {
                DBError::DuplicateTopic
            } else if msg.contains("starchart_repositories.html_url") {
                DBError::DuplicateRepositoryLink
            } else if msg.contains("starchart_forge_type.name") {
                DBError::DuplicateForgeType
            } else if msg.contains("starchart_users.html_url") {
                DBError::DuplicateUserLink
            } else if msg.contains("starchart_users.profile_photo_html_url") {
                DBError::DuplicateProfilePhotoLink
            } else if msg.contains("starchart_project_topics.name") {
                DBError::DuplicateTopic
            } else if msg.contains("starchart_repositories.name") {
                DBError::DuplicateRepositoryLink
            } else {
                DBError::DBError(Box::new(Error::Database(err)).into())
            }
        } else {
            DBError::DBError(Box::new(Error::Database(err)).into())
        }
    } else {
        DBError::DBError(Box::new(e).into())
    }
}
