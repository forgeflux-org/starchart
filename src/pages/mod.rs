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
use actix_web::*;
use lazy_static::lazy_static;
use rust_embed::RustEmbed;
use serde::*;
use tera::*;

use crate::settings::Settings;
use crate::static_assets::ASSETS;
use crate::{GIT_COMMIT_HASH, VERSION};

pub mod auth;
pub mod chart;
mod errors;
pub mod routes;

pub use errors::ERROR_KEY;
pub use routes::PAGES;

pub const TITLE_KEY: &str = "title";

pub struct TemplateFile {
    pub name: &'static str,
    pub path: &'static str,
}

impl TemplateFile {
    pub const fn new(name: &'static str, path: &'static str) -> Self {
        Self { name, path }
    }

    pub fn register(&self, t: &mut Tera) -> std::result::Result<(), tera::Error> {
        t.add_raw_template(self.name, &Templates::get_template(self).expect(self.name))
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub fn register_from_file(&self, t: &mut Tera) -> std::result::Result<(), tera::Error> {
        use std::path::Path;
        t.add_template_file(Path::new("templates/").join(self.path), Some(self.name))
    }
}

pub const PAYLOAD_KEY: &str = "payload";

pub const BASE: TemplateFile = TemplateFile::new("base", "components/base.html");
pub const FOOTER: TemplateFile = TemplateFile::new("footer", "components/footer.html");
pub const PUB_NAV: TemplateFile = TemplateFile::new("pub_nav", "components/nav/pub.html");

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();
        for t in [BASE, FOOTER, PUB_NAV ].iter() {
            t.register(&mut tera).unwrap();
        }
        errors::register_templates(&mut tera);
        auth::register_templates(&mut tera);
        chart::register_templates(&mut tera);
        tera.autoescape_on(vec![".html", ".sql"]);
        //auth::register_templates(&mut tera);
        //gists::register_templates(&mut tera);
        tera
    };
}

#[derive(RustEmbed)]
#[folder = "templates/"]
pub struct Templates;

impl Templates {
    pub fn get_template(t: &TemplateFile) -> Option<String> {
        match Self::get(t.path) {
            Some(file) => Some(String::from_utf8_lossy(&file.data).into_owned()),
            None => None,
        }
    }
}

pub fn ctx(s: &Settings) -> Context {
    let mut ctx = Context::new();
    let footer = Footer::new(s);
    ctx.insert("footer", &footer);
    ctx.insert("page", &PAGES);
    ctx.insert("assets", &*ASSETS);
    ctx
}

#[derive(Serialize)]
pub struct Footer<'a> {
    version: &'a str,
    admin_email: &'a str,
    source_code: &'a str,
    git_hash: &'a str,
    settings: &'a Settings,
}

impl<'a> Footer<'a> {
    pub fn new(settings: &'a Settings) -> Self {
        Self {
            version: VERSION,
            source_code: &settings.source_code,
            admin_email: &settings.admin_email,
            git_hash: &GIT_COMMIT_HASH[..8],
            settings,
        }
    }
}

pub fn services(cfg: &mut web::ServiceConfig) {
    auth::services(cfg);
    chart::services(cfg);
}

#[cfg(test)]
mod tests {

    #[test]
    fn templates_work_basic() {
        use super::*;
        use tera::Tera;

        let mut tera = Tera::default();
        let mut tera2 = Tera::default();
        for t in [
            BASE,
            FOOTER,
            PUB_NAV,
            auth::AUTH_CHALLENGE,
            auth::AUTH_ADD,
            chart::EXPLORE,
            //            auth::AUTH_BASE,
            //            auth::login::LOGIN,
            //            auth::register::REGISTER,
            //            errors::ERROR_TEMPLATE,
            //            gists::GIST_BASE,
            //            gists::GIST_EXPLORE,
            //            gists::new::NEW_GIST,
        ]
        .iter()
        {
            t.register_from_file(&mut tera2).unwrap();
            t.register(&mut tera).unwrap();
        }
    }
}

//#[cfg(test)]
//mod http_page_tests {
//    use actix_web::http::StatusCode;
//    use actix_web::test;
//
//    use crate::ctx::Ctx;
//    use crate::db::BoxDB;
//    use crate::tests::*;
//    use crate::*;
//
//    use super::PAGES;
//
//    #[actix_rt::test]
//    async fn sqlite_templates_work() {
//        let (db, data, _federate, _tmp_dir) = sqlx_sqlite::get_ctx().await;
//        templates_work(data, db).await;
//    }
//
//    async fn templates_work(data: ArcCtx, db: BoxDB) {
//        let app = get_app!(data, db).await;
//
//        for file in [PAGES.auth.login, PAGES.auth.register].iter() {
//            let resp = get_request!(&app, file);
//            assert_eq!(resp.status(), StatusCode::OK);
//        }
//    }
//}
