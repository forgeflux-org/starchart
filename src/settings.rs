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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Display, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
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
        self.create_license_file();
    }

    fn create_license_file(&self) {
        let root = Path::new(&self.root);
        let mut license_path = root.to_path_buf();
        license_path.push(LICENSE_FILE);
        if license_path.exists() {
            if license_path.is_dir() {
                panic!("Can't create license file at {:?}", license_path);
            } else {
                if !fs::read_to_string(&license_path)
                    .unwrap()
                    .contains(CC0_LICENSE_TXT)
                {
                    panic!("Can't create license file at {:?}", &license_path);
                }
            }
        } else {
            fs::write(license_path, CC0_LICENSE_TXT).unwrap();
        }
    }
}

#[derive(Debug, Display, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Database {
    pub url: String,
    pub pool: u32,
    pub database_type: DBType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Crawler {
    pub ttl: u64,
    pub client_timeout: u64,
    pub items_per_api_call: u64,
    pub wait_before_next_api_call: u64,
}

#[derive(Debug, Validate, Clone, PartialEq, Serialize, Deserialize)]
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

        let mut license_path = Path::new(&repo.root).to_path_buf();
        license_path.push(LICENSE_FILE);
        assert!(license_path.exists());
        assert!(license_path.is_file());
        assert!(fs::read_to_string(license_path)
            .unwrap()
            .contains(CC0_LICENSE_TXT));
    }
}

const CC0_LICENSE_TXT: &str = r#"
Creative Commons Legal Code

CC0 1.0 Universal

    CREATIVE COMMONS CORPORATION IS NOT A LAW FIRM AND DOES NOT PROVIDE
    LEGAL SERVICES. DISTRIBUTION OF THIS DOCUMENT DOES NOT CREATE AN
    ATTORNEY-CLIENT RELATIONSHIP. CREATIVE COMMONS PROVIDES THIS
    INFORMATION ON AN "AS-IS" BASIS. CREATIVE COMMONS MAKES NO WARRANTIES
    REGARDING THE USE OF THIS DOCUMENT OR THE INFORMATION OR WORKS
    PROVIDED HEREUNDER, AND DISCLAIMS LIABILITY FOR DAMAGES RESULTING FROM
    THE USE OF THIS DOCUMENT OR THE INFORMATION OR WORKS PROVIDED
    HEREUNDER.

Statement of Purpose

The laws of most jurisdictions throughout the world automatically confer
exclusive Copyright and Related Rights (defined below) upon the creator
and subsequent owner(s) (each and all, an "owner") of an original work of
authorship and/or a database (each, a "Work").

Certain owners wish to permanently relinquish those rights to a Work for
the purpose of contributing to a commons of creative, cultural and
scientific works ("Commons") that the public can reliably and without fear
of later claims of infringement build upon, modify, incorporate in other
works, reuse and redistribute as freely as possible in any form whatsoever
and for any purposes, including without limitation commercial purposes.
These owners may contribute to the Commons to promote the ideal of a free
culture and the further production of creative, cultural and scientific
works, or to gain reputation or greater distribution for their Work in
part through the use and efforts of others.

For these and/or other purposes and motivations, and without any
expectation of additional consideration or compensation, the person
associating CC0 with a Work (the "Affirmer"), to the extent that he or she
is an owner of Copyright and Related Rights in the Work, voluntarily
elects to apply CC0 to the Work and publicly distribute the Work under its
terms, with knowledge of his or her Copyright and Related Rights in the
Work and the meaning and intended legal effect of CC0 on those rights.

1. Copyright and Related Rights. A Work made available under CC0 may be
protected by copyright and related or neighboring rights ("Copyright and
Related Rights"). Copyright and Related Rights include, but are not
limited to, the following:

  i. the right to reproduce, adapt, distribute, perform, display,
     communicate, and translate a Work;
 ii. moral rights retained by the original author(s) and/or performer(s);
iii. publicity and privacy rights pertaining to a person's image or
     likeness depicted in a Work;
 iv. rights protecting against unfair competition in regards to a Work,
     subject to the limitations in paragraph 4(a), below;
  v. rights protecting the extraction, dissemination, use and reuse of data
     in a Work;
 vi. database rights (such as those arising under Directive 96/9/EC of the
     European Parliament and of the Council of 11 March 1996 on the legal
     protection of databases, and under any national implementation
     thereof, including any amended or successor version of such
     directive); and
vii. other similar, equivalent or corresponding rights throughout the
     world based on applicable law or treaty, and any national
     implementations thereof.

2. Waiver. To the greatest extent permitted by, but not in contravention
of, applicable law, Affirmer hereby overtly, fully, permanently,
irrevocably and unconditionally waives, abandons, and surrenders all of
Affirmer's Copyright and Related Rights and associated claims and causes
of action, whether now known or unknown (including existing as well as
future claims and causes of action), in the Work (i) in all territories
worldwide, (ii) for the maximum duration provided by applicable law or
treaty (including future time extensions), (iii) in any current or future
medium and for any number of copies, and (iv) for any purpose whatsoever,
including without limitation commercial, advertising or promotional
purposes (the "Waiver"). Affirmer makes the Waiver for the benefit of each
member of the public at large and to the detriment of Affirmer's heirs and
successors, fully intending that such Waiver shall not be subject to
revocation, rescission, cancellation, termination, or any other legal or
equitable action to disrupt the quiet enjoyment of the Work by the public
as contemplated by Affirmer's express Statement of Purpose.

3. Public License Fallback. Should any part of the Waiver for any reason
be judged legally invalid or ineffective under applicable law, then the
Waiver shall be preserved to the maximum extent permitted taking into
account Affirmer's express Statement of Purpose. In addition, to the
extent the Waiver is so judged Affirmer hereby grants to each affected
person a royalty-free, non transferable, non sublicensable, non exclusive,
irrevocable and unconditional license to exercise Affirmer's Copyright and
Related Rights in the Work (i) in all territories worldwide, (ii) for the
maximum duration provided by applicable law or treaty (including future
time extensions), (iii) in any current or future medium and for any number
of copies, and (iv) for any purpose whatsoever, including without
limitation commercial, advertising or promotional purposes (the
"License"). The License shall be deemed effective as of the date CC0 was
applied by Affirmer to the Work. Should any part of the License for any
reason be judged legally invalid or ineffective under applicable law, such
partial invalidity or ineffectiveness shall not invalidate the remainder
of the License, and in such case Affirmer hereby affirms that he or she
will not (i) exercise any of his or her remaining Copyright and Related
Rights in the Work or (ii) assert any associated claims and causes of
action with respect to the Work, in either case contrary to Affirmer's
express Statement of Purpose.

4. Limitations and Disclaimers.

 a. No trademark or patent rights held by Affirmer are waived, abandoned,
    surrendered, licensed or otherwise affected by this document.
 b. Affirmer offers the Work as-is and makes no representations or
    warranties of any kind concerning the Work, express, implied,
    statutory or otherwise, including without limitation warranties of
    title, merchantability, fitness for a particular purpose, non
    infringement, or the absence of latent or other defects, accuracy, or
    the present or absence of errors, whether or not discoverable, all to
    the greatest extent permissible under applicable law.
 c. Affirmer disclaims responsibility for clearing rights of other persons
    that may apply to the Work or any use thereof, including without
    limitation any person's Copyright and Related Rights in the Work.
    Further, Affirmer disclaims responsibility for obtaining any necessary
    consents, permissions or other rights required for any use of the
    Work.
 d. Affirmer understands and acknowledges that Creative Commons is not a
    party to this document and has no duty or obligation with respect to
    this CC0 or use of the Work.
"#;

const LICENSE_FILE: &str = "LICENSE.txt";
