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
use serde::Deserialize;
use url::Url;
use validator::Validate;

#[derive(Debug, Clone, Deserialize)]
pub struct Server {
    pub port: u32,
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

#[derive(Deserialize, Display, Clone, Debug)]
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

#[derive(Debug, Clone, Deserialize)]
pub struct Repository {
    pub root: String,
}

impl Repository {
    fn create_root_dir(&self) {
        let root = Path::new(&self.root);
        if root.exists() {
            if !root.is_dir() {
                fs::remove_file(&root).unwrap();
                fs::create_dir_all(&root).unwrap();
            }
        } else {
            fs::create_dir_all(&root).unwrap();
        }
    }
}

#[derive(Deserialize, Display, PartialEq, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum DBType {
    #[display(fmt = "postgres")]
    Postgres,
    #[display(fmt = "sqlite")]
    Sqlite,
}

impl DBType {
    fn from_url(url: &Url) -> Result<Self, ConfigError> {
        match url.scheme() {
            "sqlite" => Ok(Self::Sqlite),
            "postgres" => Ok(Self::Postgres),
            _ => Err(ConfigError::Message("Unknown database type".into())),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct DatabaseBuilder {
    pub port: u32,
    pub hostname: String,
    pub username: String,
    pub password: String,
    pub name: String,
    pub database_type: DBType,
}

impl DatabaseBuilder {
    #[cfg(not(tarpaulin_include))]
    fn extract_database_url(url: &Url) -> Self {
        log::debug!("Databse name: {}", url.path());
        let mut path = url.path().split('/');
        path.next();
        let name = path.next().expect("no database name").to_string();

        let database_type = DBType::from_url(url).unwrap();
        let port = if database_type == DBType::Sqlite {
            0
        } else {
            url.port().expect("Enter database port").into()
        };

        DatabaseBuilder {
            port,
            hostname: url.host().expect("Enter database host").to_string(),
            username: url.username().into(),
            password: url.password().expect("Enter database password").into(),
            name,
            database_type: DBType::from_url(url).unwrap(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Database {
    pub url: String,
    pub pool: u32,
    pub database_type: DBType,
}

#[derive(Debug, Validate, Clone, Deserialize)]
pub struct Crawler {
    pub ttl: u64,
    pub client_timeout: u64,
    pub items_per_api_call: u64,
    pub wait_before_next_api_call: u64,
}

#[derive(Debug, Validate, Clone, Deserialize)]
pub struct Settings {
    pub log: LogLevel,
    pub database: Database,
    pub allow_new_index: bool,
    pub server: Server,
    #[validate(url)]
    pub source_code: String,
    pub repository: Repository,
    #[validate(email)]
    pub admin_email: String,
    pub crawler: Crawler,
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
        const ETC: &str = "/etc/starchart/config.toml";

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

        if let Ok(path) = env::var("STARCHART_CONFIG") {
            s = s.add_source(File::with_name(&path));
            read_file = true;
        }
        if !read_file {
            log::warn!("configuration file not found");
        }

        s = s.add_source(Environment::with_prefix("STARCHART").separator("__"));

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
        settings.repository.create_root_dir();
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

//#[cfg(not(tarpaulin_include))]
//fn set_from_database_url(s: &mut Config, database_conf: &DatabaseBuilder) {
//    s.set("database.username", database_conf.username.clone())
//        .expect("Couldn't set database username");
//    s.set("database.password", database_conf.password.clone())
//        .expect("Couldn't access database password");
//    s.set("database.hostname", database_conf.hostname.clone())
//        .expect("Couldn't access database hostname");
//    s.set("database.port", database_conf.port as i64)
//        .expect("Couldn't access database port");
//    s.set("database.name", database_conf.name.clone())
//        .expect("Couldn't access database name");
//    s.set(
//        "database.database_type",
//        format!("{}", database_conf.database_type),
//    )
//    .expect("Couldn't access database type");
//}

//#[cfg(not(tarpaulin_include))]
//fn set_database_url(s: &mut Config) {
//    s.set(
//        "database.url",
//        format!(
//            r"{}://{}:{}@{}:{}/{}",
//            s.get::<String>("database.database_type")
//                .expect("Couldn't access database database_type"),
//            s.get::<String>("database.username")
//                .expect("Couldn't access database username"),
//            s.get::<String>("database.password")
//                .expect("Couldn't access database password"),
//            s.get::<String>("database.hostname")
//                .expect("Couldn't access database hostname"),
//            s.get::<String>("database.port")
//                .expect("Couldn't access database port"),
//            s.get::<String>("database.name")
//                .expect("Couldn't access database name")
//        ),
//    )
//    .expect("Couldn't set databse url");
//}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::get_random;

    #[test]
    fn database_type_test() {
        for i in ["sqlite://foo", "postgres://bar", "unknown://"].iter() {
            let url = Url::parse(i).unwrap();
            if i.contains("sqlite") {
                assert_eq!(DBType::from_url(&url).unwrap(), DBType::Sqlite);
            } else if i.contains("unknown") {
                assert!(DBType::from_url(&url).is_err());
            } else {
                assert_eq!(DBType::from_url(&url).unwrap(), DBType::Postgres);
            }
        }
    }

    #[test]
    fn root_dir_is_created_test() {
        let dir;
        loop {
            let mut tmp = env::temp_dir();
            tmp = tmp.join(get_random(10));

            if tmp.exists() {
                continue;
            } else {
                dir = tmp;
                break;
            }
        }

        let repo = Repository {
            root: dir.to_str().unwrap().to_owned(),
        };

        repo.create_root_dir();
        assert!(dir.exists());
        assert!(dir.is_dir());
        let file = dir.join("foo");
        fs::write(&file, "foo").unwrap();
        repo.create_root_dir();
        assert!(dir.exists());
        assert!(dir.is_dir());

        assert!(file.exists());
        assert!(file.is_file());

        let repo = Repository {
            root: file.to_str().unwrap().to_owned(),
        };

        repo.create_root_dir();
        assert!(file.exists());
        assert!(file.is_dir());
    }
}
