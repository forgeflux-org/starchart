/*
 * Copyright (C) 2021  Aravinth Manivannan <realaravinth@batsense.net>
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
use std::process::Command;

//use cache_buster::{BusterBuilder, NoHashCategory};

fn main() {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("error in git command, is git installed?");
    let git_hash = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);

    //    cache_bust();
}

//fn cache_bust() {
//    //    until APPLICATION_WASM gets added to mime crate
//    //    PR: https://github.com/hyperium/mime/pull/138
//    //    let types = vec![
//    //        mime::IMAGE_PNG,
//    //        mime::IMAGE_SVG,
//    //        mime::IMAGE_JPEG,
//    //        mime::IMAGE_GIF,
//    //        mime::APPLICATION_JAVASCRIPT,
//    //        mime::TEXT_CSS,
//    //    ];
//
//    println!("cargo:rerun-if-changed=static/cache");
//    let no_hash = vec![NoHashCategory::FileExtentions(vec!["wasm"])];
//
//    let config = BusterBuilder::default()
//        .source("./static/cache/")
//        .result("./assets")
//        .no_hash(no_hash)
//        .follow_links(true)
//        .build()
//        .unwrap();
//
//    config.process().unwrap();
//}
