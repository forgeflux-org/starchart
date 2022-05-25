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
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct Configuration {
    pub spidering: bool,
    pub rate: Option<u64>,
}

impl Configuration {
    pub fn parse(s: &str) -> Self {
        fn parse_inner(config: &mut Configuration, s: &str) {
            let mut inner = s.split('=');
            let k = inner.next().unwrap().trim();
            let v = inner.next().unwrap().trim();
            println!("split inner: {:?}: {:?}", k, v);

            if k == "spidering" {
                if v == "false" {
                    config.spidering = false;
                } else if v == "true" {
                    config.spidering = true;
                } else {
                    panic!("Value {k} is not bool, can't set for spidering");
                }
            } else if k == "rate" {
                let x: u64 = v.parse().unwrap();
                config.rate = Some(x);
            } else {
                panic!("Key {k} and Value {v} is implemented or supported");
            }
        }
        let mut config = Self::default();
        if s.contains(',') {
            for spilt in s.split(',') {
                println!("split: {:?}", spilt);
                parse_inner(&mut config, spilt);
            }
        } else {
            parse_inner(&mut config, s);
        }
        config
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dns_txt_parser_works() {
        const REQ: &str = "spidering=false,rate=500";
        const RES: Configuration = Configuration {
            spidering: false,
            rate: Some(500),
        };

        const REQ_2: &str = "spidering=true";
        const RES_2: Configuration = Configuration {
            spidering: true,
            rate: None,
        };

        assert_eq!(Configuration::parse(REQ), RES);
        assert_eq!(Configuration::parse(REQ_2), RES_2);
    }
}
