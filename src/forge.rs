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
use async_trait::async_trait;
use db_core::prelude::*;

#[async_trait]
pub trait SCForge: std::marker::Send + std::marker::Sync + CloneSPForge {
    async fn is_forge(&self) -> bool;
    async fn get_repositories(&self, limit: usize, page: usize) -> Vec<AddRepository>;
}

/// Trait to clone SCForge
pub trait CloneSPForge {
    /// clone DB
    fn clone_db(&self) -> Box<dyn SCForge>;
}

impl<T> CloneSPForge for T
where
    T: SCForge + Clone + 'static,
{
    fn clone_db(&self) -> Box<dyn SCForge> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn SCForge> {
    fn clone(&self) -> Self {
        (**self).clone_db()
    }
}
