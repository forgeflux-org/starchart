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

use actix_files::Files;
use actix_web::{middleware, web::Data, App, HttpServer};
use lazy_static::lazy_static;

pub mod ctx;
pub mod db;
pub mod dns;
pub mod errors;
pub mod federate;
pub mod forge;
pub mod pages;
pub mod routes;
pub mod settings;
pub mod spider;
pub mod static_assets;
#[cfg(test)]
mod tests;
pub mod utils;
pub mod verify;

use crate::federate::{get_federate, ArcFederate};
use ctx::Ctx;
use db::{sqlite, BoxDB};
use settings::Settings;
use static_assets::FileMap;

pub use crate::pages::routes::PAGES;

pub const CACHE_AGE: u32 = 60 * 60 * 24 * 30; // one month, I think?
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
pub const GIT_COMMIT_HASH: &str = env!("GIT_HASH");
pub const DOMAIN: &str = "developer-starchart.forgeflux.org";

pub type ArcCtx = Arc<Ctx>;
pub type WebCtx = Data<ArcCtx>;
pub type WebDB = Data<BoxDB>;
pub type WebFederate = Data<ArcFederate>;

lazy_static! {
    pub static ref FILES: FileMap = FileMap::new();
}

#[actix_rt::main]
async fn main() {
    let settings = Settings::new().unwrap();
    pretty_env_logger::init();
    lazy_static::initialize(&pages::TEMPLATES);

    let ctx = WebCtx::new(Ctx::new(settings.clone()).await);
    let db = WebDB::new(sqlite::get_data(Some(settings.clone())).await);
    let federate = WebFederate::new(get_federate(Some(settings.clone())).await);

    let socket_addr = settings.server.get_ip();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .app_data(federate.clone())
            .app_data(db.clone())
            .app_data(ctx.clone())
            .wrap(
                middleware::DefaultHeaders::new().add(("Permissions-Policy", "interest-cohort=()")),
            )
            .configure(routes::services)
            .service(Files::new("/federate", &settings.repository.root).show_files_listing())
    })
    .bind(&socket_addr)
    .unwrap()
    .run()
    .await
    .unwrap();
}
