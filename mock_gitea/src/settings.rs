/*
 * ForgeFlux StarChart - A federated software forge spider
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
use std::path::Path;
use std::{env, fs};

use config::{Config, ConfigError, Environment, File};
use derive_more::Display;
use log::warn;
use serde::{Deserialize, Serialize};
use url::Url;
use validator::Validate;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Server {
    pub port: u32,
    pub workers: usize,
    pub domain: String,
    pub ip: String,
    pub proxy_has_tls: bool,
}

impl Server {
    #[cfg(not(tarpaulin_include))]
    pub fn get_ip(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

#[derive(Debug, Display, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    #[display(fmt = "debug")]
    Debug,
    #[display(fmt = "info")]
    Info,
    #[display(fmt = "trace")]
    Trace,
    #[display(fmt = "error")]
    Error,
    #[display(fmt = "warn")]
    Warn,
}

impl LogLevel {
    fn set_log_level(&self) {
        const LOG_VAR: &str = "RUST_LOG";
        if env::var(LOG_VAR).is_err() {
            env::set_var("RUST_LOG", format!("{}", self));
        }
    }
}

#[derive(Debug, Display, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DBType {
    #[display(fmt = "postgres")]
    Postgres,
    #[display(fmt = "sqlite")]
    Sqlite,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Database {
    pub url: String,
    pub pool: u32,
    pub database_type: DBType,
}

#[derive(Debug, Validate, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    pub log: LogLevel,
    pub database: Database,
    pub server: Server,
    #[validate(url)]
    pub source_code: String,
    #[validate(email)]
    pub admin_email: String,
    pub data: String,
}

#[cfg(not(tarpaulin_include))]
impl Settings {
    fn set_source_code(&mut self) {
        if !self.source_code.ends_with('/') {
            self.source_code.push('/');
        }
        let mut base = url::Url::parse(&self.source_code).unwrap();
        base = base.join("tree/").unwrap();
        base = base.join(crate::GIT_COMMIT_HASH).unwrap();
        self.source_code = base.into();
    }

    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::builder();

        // setting default values
        #[cfg(test)]
        {
            s = s
                .set_default("database.pool", "2")
                .expect("Couldn't get the number of CPUs");
        }

        const CURRENT_DIR: &str = "./config/default.toml";
        const ETC: &str = "/etc/mock_gitea/config.toml";

        let mut read_file = false;
        if Path::new(CURRENT_DIR).exists() {
            // merging default config from file
            s = s.add_source(File::with_name(CURRENT_DIR));
            read_file = true;
        }

        if Path::new(ETC).exists() {
            s = s.add_source(File::with_name(ETC));
            read_file = true;
        }

        if let Ok(path) = env::var("MGITEA_CONFIG") {
            s = s.add_source(File::with_name(&path));
            read_file = true;
        }
        if !read_file {
            log::warn!("configuration file not found");
        }

        s = s.add_source(Environment::with_prefix("MGITEA").separator("__"));

        match env::var("PORT") {
            Ok(val) => s = s.set_override("server.port", val).unwrap(),
            Err(e) => warn!("couldn't interpret PORT: {}", e),
        }

        match env::var("DATABASE_URL") {
            Ok(val) => s = s.set_override("database.url", val).unwrap(),
            //let url = Url::parse(&val).expect("couldn't parse Database URL");
            //TODO: sqlite fails Url::parse, figure out workarounds
            Err(e) => {
                warn!("couldn't interpret DATABASE_URL: {}", e);
            }
        }

        let mut settings = s.build()?.try_deserialize::<Settings>()?;
        settings.check_url();

        settings.log.set_log_level();
        settings.validate().unwrap();
        settings.set_source_code();
        settings.validate().unwrap();

        Ok(settings)
    }

    #[cfg(not(tarpaulin_include))]
    fn check_url(&self) {
        Url::parse(&self.source_code).expect("Please enter a URL for source_code in settings");
    }
}
