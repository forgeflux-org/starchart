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

use actix::dev::*;
use reqwest::{Client, ClientBuilder};

use crate::master::Master;
use crate::settings::Settings;
use crate::{PKG_NAME, VERSION};

/// in seconds
const CLIENT_TIMEOUT: u64 = 60;

#[derive(Clone)]
pub struct Ctx {
    pub client: Client,
    pub settings: Settings,
    pub master: Addr<Master>,
}

impl Ctx {
    pub async fn new(settings: Settings) -> Arc<Self> {
        let host = settings.introducer.public_url.host_str().unwrap();
        let host = if let Some(port) = settings.introducer.public_url.port() {
            format!("{host}:{port}")
        } else {
            host.to_owned()
        };
        let ua = format!("{VERSION}---{PKG_NAME}---{host}");
        let timeout = Duration::new(CLIENT_TIMEOUT, 0);
        let client = ClientBuilder::new()
            .user_agent(&*ua)
            .use_rustls_tls()
            .timeout(timeout)
            .connect_timeout(timeout)
            .tcp_keepalive(timeout)
            .build()
            .unwrap();

        let master = Master::new(45).start();

        Arc::new(Self {
            client,
            settings,
            master,
        })
    }
}
