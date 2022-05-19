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
use trust_dns_resolver::{
    config::{ResolverConfig, ResolverOpts},
    AsyncResolver,
};
use lazy_static::lazy_static;

use crate::utils::get_random;
use crate::ArcCtx;

pub struct TXTChallenge {
    key: String,
    _base_hostname: String,
    value: String,
}

const KEY_LEN: usize = 30;
const VALUES_LEN: usize = 30;

impl TXTChallenge {
    pub fn get_challenge_txt_key_prefix(ctx: &ArcCtx) -> String {
        // starchart-{{ starchart instance's hostname}}.{{ forge instance's hostname }}
        format!("starchart-{}", &ctx.settings.server.domain)
    }

    pub fn get_challenge_txt_key(ctx: &ArcCtx, hostname: &str) -> String {
        format!("{}.{}", Self::get_challenge_txt_key_prefix(ctx), hostname)
    }



    pub async fn new(ctx: &ArcCtx, hostname: &str) -> Self {
        let key = Self::get_challenge_txt_key(ctx, hostname);
        let value = get_random(VALUES_LEN);
        Self {
            key,
            value,
            _base_hostname: hostname.to_string(),
        }
    }

    pub async fn verify_txt(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let conf = ResolverConfig::cloudflare_tls();
        let opts = ResolverOpts::default();
        let resolver = AsyncResolver::tokio(conf, opts)?;
        let res = resolver.txt_lookup(&self.key).await?;
        Ok(res.iter().any(|r| r.to_string() == self.value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::sqlx_sqlite;

    #[actix_rt::test]
    async fn verify_txt_works() {
        // please note that this DNS record is in prod
        const BASE_DOMAIN: &str = "forge.forgeflux.org";
        const VALUE: &str = "ifthisvalueisretrievedbyforgefluxstarchartthenthetestshouldpass";

        let (_db, ctx, _federate, _tmp_dir) = sqlx_sqlite::get_ctx().await;

        let key =  TXTChallenge::get_challenge_txt_key(&ctx, BASE_DOMAIN);
        let mut txt_challenge = TXTChallenge {
            value: VALUE.to_string(),
            _base_hostname: BASE_DOMAIN.to_string(),
            key: key.clone(),
        };
        assert_eq!(
            TXTChallenge::get_challenge_txt_key(&ctx, BASE_DOMAIN),
            key,
        );

        assert!(
            txt_challenge.verify_txt().await.unwrap(),
            "TXT Challenge verification test"
        );
        txt_challenge.value = key;
        assert!(!txt_challenge.verify_txt().await.unwrap());
    }
}
