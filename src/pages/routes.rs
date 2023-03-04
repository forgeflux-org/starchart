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
use serde::Serialize;

/// constant [Pages](Pages) instance
pub const PAGES: Pages = Pages::new();

#[derive(Serialize)]
/// Top-level routes data structure for V1 AP1
pub struct Pages {
    /// home page
    pub home: &'static str,
    pub explore: &'static str,
    pub search: &'static str,
    /// auth routes
    pub auth: Auth,
}

impl Pages {
    /// create new instance of Routes
    const fn new() -> Pages {
        let explore = "/explore";
        let home = explore;
        let search = "/search";
        let auth = Auth::new();
        Pages {
            home,
            auth,
            explore,
            search,
        }
    }

    pub fn explore_next(&self, page: u32) -> String {
        format!("{}?page={page}", self.explore)
    }
}

#[derive(Serialize)]
/// Authentication routes
pub struct Auth {
    /// logout route
    pub logout: &'static str,
    /// login route
    pub add: &'static str,

    /// verify route
    pub verify: &'static str,
}

impl Auth {
    /// create new instance of Authentication route
    pub const fn new() -> Auth {
        let add = "/add";
        let logout = "/logout";
        let verify = "/verify";
        Auth {
            add,
            logout,
            verify,
        }
    }

    pub fn verify_get(&self, hostname: &str) -> String {
        format!("{}?hostname={hostname}", self.verify)
    }
}

//#[cfg(test)]
//mod tests {
//    use super::*;
//    #[test]
//    fn gist_route_substitution_works() {
//        const NAME: &str = "bob";
//        const GIST: &str = "foo";
//        const FILE: &str = "README.md";
//        let get_profile = format!("/~{NAME}");
//        let view_gist = format!("/~{NAME}/{GIST}");
//        let post_comment = format!("/~{NAME}/{GIST}/comment");
//        let get_file = format!("/~{NAME}/{GIST}/contents/{FILE}");
//
//        let profile_component = GistProfilePathComponent { username: NAME };
//
//        assert_eq!(get_profile, PAGES.gist.get_profile_route(profile_component));
//
//        let profile_component = PostCommentPath {
//            username: NAME.into(),
//            gist: GIST.into(),
//        };
//
//        assert_eq!(view_gist, PAGES.gist.get_gist_route(&profile_component));
//
//        let post_comment_path = PostCommentPath {
//            gist: GIST.into(),
//            username: NAME.into(),
//        };
//
//        assert_eq!(
//            post_comment,
//            PAGES.gist.get_post_comment_route(&post_comment_path)
//        );
//
//        let file_component = GetFilePath {
//            username: NAME.into(),
//            gist: GIST.into(),
//            file: FILE.into(),
//        };
//        assert_eq!(get_file, PAGES.gist.get_file_route(&file_component));
//    }
//}
