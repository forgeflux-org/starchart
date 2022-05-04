/*
 * ForgeFlux StarChart - A federated software forge spider
 * Copyright © 2usize22 Aravinth Manivannan <realaravinth@batsense.net>
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
use std::collections::HashMap;

use db_core::AddRepository;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResults {
    pub ok: bool,
    pub data: Vec<Repository>,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: usize,
    pub login: String,
    pub full_name: String,
    pub email: String,
    pub avatar_url: String,
    pub language: String,
    pub is_admin: bool,
    pub last_login: String,
    pub created: String,
    pub restricted: bool,
    pub active: bool,
    pub prohibit_login: bool,
    pub location: String,
    pub website: String,
    pub description: String,
    pub visibility: String,
    pub followers_count: usize,
    pub following_count: usize,
    pub starred_repos_count: usize,
    pub username: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Repository {
    pub name: String,
    pub full_name: String,
    pub description: String,
    pub empty: bool,
    pub private: bool,
    pub fork: bool,
    pub template: bool,
    pub parent: Option<Box<Repository>>,
    pub mirror: bool,
    pub size: usize,
    pub html_url: String,
    pub ssh_url: String,
    pub clone_url: String,
    pub original_url: String,
    pub owner: User,
    pub website: String,
    pub stars_count: usize,
    pub forks_count: usize,
    pub watchers_count: usize,
    pub open_issues_count: usize,
    pub open_pr_counter: usize,
    pub release_counter: usize,
    pub default_branch: String,
    pub archived: bool,
    pub created_at: String,
    pub updated_at: String,
    pub internal_tracker: InternalIssueTracker,
    pub has_issues: bool,
    pub has_wiki: bool,
    pub has_pull_requests: bool,
    pub has_projects: bool,
    pub ignore_whitespace_conflicts: bool,
    pub allow_merge_commits: bool,
    pub allow_rebase: bool,
    pub allow_rebase_explicit: bool,
    pub allow_squash_merge: bool,
    pub default_merge_style: String,
    pub avatar_url: String,
    pub internal: bool,
    pub mirror_interval: String,
    pub mirror_updated: String,
    pub repo_transfer: Option<Team>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InternalIssueTracker {
    pub enable_time_tracker: bool,
    pub allow_only_contributors_to_track_time: bool,
    pub enable_issue_dependencies: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepoTransfer {
    pub doer: User,
    pub recipient: User,
    pub teams: Option<Team>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Hash, Deserialize)]
pub struct Organization {
    pub avatar_url: String,
    pub description: String,
    pub full_name: String,
    pub id: u64,
    pub location: String,
    pub repo_admin_change_team_access: bool,
    pub username: String,
    pub visibility: String,
    pub website: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Permission {
    None,
    Read,
    Write,
    Admin,
    Owner,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Team {
    pub can_create_org_repo: bool,
    pub description: String,
    pub id: u64,
    pub includes_all_repositories: bool,
    pub name: String,
    pub organization: Organization,
    pub permission: Permission,
    pub units: Vec<String>,
    pub units_map: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Topics {
    pub topics: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    #[test]
    /// Tests if Gitea responses panic when deserialized with serde into structs defined in this
    /// module/file. Since Go doesn't have abilities to describe nullable values, I(@realaravinth)
    /// am forced to do this as I my knowledge about Gitea codebase is very limited.
    fn schema_doesnt_panic() {
        let files = ["./tests/schema/gitea/git.batsense.net.json"];
        for file in files.iter() {
            let contents = fs::read_to_string(file).unwrap();
            for line in contents.lines() {
                let _: SearchResults = serde_json::from_str(line).expect("Gitea schema paniced");
            }
        }
    }
}
