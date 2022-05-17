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
use std::sync::Arc;
use std::time::Duration;

use lazy_static::lazy_static;
use reqwest::{Client, ClientBuilder};

use crate::settings::Settings;
use crate::{DOMAIN, PKG_NAME, VERSION};

lazy_static! {
    pub static ref USER_AGENT: String = format!("{VERSION}---{PKG_NAME}---{DOMAIN}");
}
/// in seconds
const CLIENT_TIMEOUT: u64 = 60;

#[derive(Clone)]
pub struct Ctx {
    pub client: Client,
    pub settings: Settings,
}

impl Ctx {
    pub async fn new(settings: Settings) -> Arc<Self> {
        let timeout = Duration::new(CLIENT_TIMEOUT, 0);
        let client = ClientBuilder::new()
            .user_agent(&*USER_AGENT)
            .use_rustls_tls()
            .timeout(timeout)
            .connect_timeout(timeout)
            .tcp_keepalive(timeout)
            .build()
            .unwrap();

        Arc::new(Self { client, settings })
    }
}
