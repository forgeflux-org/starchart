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

pub mod home;
pub mod search;
pub use home::EXPLORE;
pub use home::REPO_INFO;
pub use home::SEARCH_BAR;
pub use search::SEARCH_RESULTS;

pub use super::{ctx, TemplateFile, ERROR_KEY, PAGES, PAYLOAD_KEY, TITLE_KEY};

pub fn register_templates(t: &mut tera::Tera) {
    EXPLORE.register(t).expect(EXPLORE.name);
    REPO_INFO.register(t).expect(REPO_INFO.name);
    SEARCH_BAR.register(t).expect(SEARCH_BAR.name);
    SEARCH_RESULTS.register(t).expect(SEARCH_RESULTS.name);
}

pub fn services(cfg: &mut actix_web::web::ServiceConfig) {
    home::services(cfg);
    search::services(cfg);
}
