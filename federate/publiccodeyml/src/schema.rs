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
use std::collections::HashMap;

use db_core::AddRepository;
use serde::{Deserialize, Serialize};
use url::Url;

const PUBLIC_CODE_VERSION: &str = "0.2";

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Repository {
    pub publiccode_yml_version: String,
    pub name: String,
    pub url: Url,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub landing_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_based_on: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub description: HashMap<String, Description>,
    pub legal: Legal,
    #[serde(skip_serializing_if = "IntendedAudience::is_none")]
    pub intended_audience: IntendedAudience,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Description {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Legal {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    pub repo_owner: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Maintenance {
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename(serialize = "type", deserialize = "m_type")
    )]
    pub m_type: Option<String>,
    pub contacts: Vec<Contacts>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Contacts {
    pub name: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntendedAudience {
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename(serialize = "type", deserialize = "m_type")
    )]
    pub scope: Option<Vec<String>>,
}

impl IntendedAudience {
    /// global is_none, to skip_serializing_if
    pub fn is_none(&self) -> bool {
        if self.scope.is_none() {
            true
        } else {
            self.scope.as_ref().unwrap().is_empty()
        }
    }
}

impl From<&db_core::AddRepository<'_>> for Repository {
    fn from(r: &db_core::AddRepository<'_>) -> Self {
        let mut description = HashMap::with_capacity(1);
        description.insert(
            "en".into(),
            Description {
                short_description: r.description.map(|d| d.into()),
                documentation: r.website.map(|d| d.into()),
                long_description: None,
            },
        );

        let legal = Legal {
            license: None,
            repo_owner: r.owner.to_string(),
        };

        let scope = r
            .tags
            .as_ref()
            .map(|tags| tags.iter().map(|t| t.to_string()).collect());
        let intended_audience = IntendedAudience { scope };

        Self {
            publiccode_yml_version: PUBLIC_CODE_VERSION.into(),
            url: Url::parse(r.html_link).unwrap(),
            landing_url: r.website.map(|s| Url::parse(s).unwrap()),
            name: r.name.into(),
            is_based_on: None, // TODO collect is_fork information in forge/*
            description,
            legal,
            intended_audience,
        }
    }
}

impl Repository {
    pub fn to_add_repository(&self, import: bool) -> AddRepository {
        let tags = self
            .intended_audience
            .scope
            .as_ref()
            .map(|s| s.iter().map(|t| t.as_str()).collect());
        let description = self
            .description
            .get("en")
            .as_ref()
            .unwrap()
            .short_description
            .as_ref()
            .map(|s| s.as_str());
        let website = self
            .description
            .get("en")
            .unwrap()
            .documentation
            .as_ref()
            .map(|s| s.as_str());
        AddRepository {
            html_link: self.url.as_str(),
            tags,
            url: self.url.clone(),
            name: &self.name,
            owner: &self.legal.repo_owner,
            description,
            website,
            import,
        }
    }
}
