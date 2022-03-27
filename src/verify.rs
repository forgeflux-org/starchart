/*
 * ForgeFlux World - A federated software forge spider
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
use trust_dns_resolver::{
    config::{ResolverConfig, ResolverOpts},
    AsyncResolver,
};

use crate::utils::get_random;

pub struct TXTChallenge {
    key: String,
    base_hostname: String,
    value: String,
}

const KEY_LEN: usize = 30;
const VALUES_LEN: usize = 30;

impl TXTChallenge {
    pub async fn new(hostname: &str) -> Self {
        let key = get_random(KEY_LEN);
        let value = get_random(VALUES_LEN);
        Self {
            key,
            value,
            base_hostname: hostname.to_string(),
        }
    }

    pub fn get_txt_key(&self) -> String {
        format!("{}.{}", self.key, self.base_hostname)
    }

    pub async fn verify_txt(&self) -> bool {
        let conf = ResolverConfig::cloudflare_tls();
        let opts = ResolverOpts::default();
        let resolver = AsyncResolver::tokio(conf, opts).unwrap();
        let res = resolver.txt_lookup(&self.get_txt_key()).await.unwrap();
        res.iter().any(|r| r.to_string() == self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn verify_txt_works() {
        // please note that this DNS record is in prod
        const KEY: &str = "test-world-foobardontmindme";
        const BASE_DOMAIN: &str = "world.forgeflux.org";
        const VALUE: &str = "ifthisvalueisretrievedbyforgefluxworldthenthetestshouldpass";
        let mut txt_challenge = TXTChallenge {
            value: VALUE.to_string(),
            base_hostname: BASE_DOMAIN.to_string(),
            key: KEY.to_string(),
        };
        assert_eq!(
            txt_challenge.get_txt_key(),
            "test-world-foobardontmindme.world.forgeflux.org",
        );

        assert!(
            txt_challenge.verify_txt().await,
            "TXT Challenge verification test"
        );
        txt_challenge.value = KEY.into();
        assert!(!txt_challenge.verify_txt().await);
    }
}
