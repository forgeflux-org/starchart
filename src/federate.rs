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
use std::sync::Arc;

use federate_core::Federate;
use publiccodeyml::{errors::FederateErorr, PccFederate};

use crate::settings::Settings;

pub type ArcFederate = Arc<dyn Federate<Error = FederateErorr>>;

pub async fn get_federate(settings: Option<Settings>) -> ArcFederate {
    let settings = settings.unwrap_or_else(|| Settings::new().unwrap());
    Arc::new(PccFederate::new(settings.repository.root).await.unwrap())
}
