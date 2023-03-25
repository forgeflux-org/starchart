/*
 * ForgeFlux StarChart - A federated software forge spider
 * Copyright © 2022 Aravinth Manivannan <realaravinth@batsense.net>
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
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */
use log::info;

use crate::ctx::Ctx;
use crate::db::{AddRepository, Database, LogInitialization};
use crate::errors::ServiceResult;
use crate::utils;

impl Ctx {
    pub async fn init(&self, db: &Database) -> ServiceResult<()> {
        log::info!("Reading repository seed data from {}", self.settings.data);
        let txt = tokio::fs::read_to_string(&self.settings.data)
            .await
            .unwrap();

        let txt = utils::TextProcessor::new(txt);
        if !db.get_initializations().await?.is_empty() {
            return Ok(());
        }

        let num_repos = 10;
        let num_tags: i64 = 20;
        let num_users = 100;

        let mut tags = Vec::with_capacity(num_tags.try_into().unwrap());
        for _ in 0..num_tags {
            tags.push(txt.get_random_shakespeare())
        }

        for i in 0..num_users {
            let username = txt.get_random_shakespeare();
            info!("[{i}] Creating user {username}");
            for _ in 0..num_repos {
                let repo_name = txt.get_random_shakespeare();
                info!("[{i}][{username}] Creating repository {repo_name}");
                let tags = [
                    *tags.get(utils::get_random_number(num_tags)).unwrap(),
                    *tags.get(utils::get_random_number(num_tags)).unwrap(),
                    *tags.get(utils::get_random_number(num_tags)).unwrap(),
                    *tags.get(utils::get_random_number(num_tags)).unwrap(),
                    *tags.get(utils::get_random_number(num_tags)).unwrap(),
                ];

                let msg = AddRepository {
                    username,
                    name: repo_name,
                    tags: &tags,
                };

                db.add_repository(msg).await?;
            }
        }

        let log = LogInitialization {
            num_repos,
            num_tags,
            num_users,
        };

        db.log_initialization(log).await?;

        Ok(())
    }
}
