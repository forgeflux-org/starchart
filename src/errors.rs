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

use std::convert::From;

use actix_web::{
    error::ResponseError,
    http::{header, StatusCode},
    HttpResponse, HttpResponseBuilder,
};
use db_core::errors::DBError;
use derive_more::{Display, Error};
use serde::{Deserialize, Serialize};
use url::ParseError;
use validator::ValidationErrors;

#[derive(Debug, Display, Error)]
pub struct DBErrorWrapper(DBError);

impl std::cmp::PartialEq for DBErrorWrapper {
    fn eq(&self, other: &Self) -> bool {
        format!("{}", self.0) == format!("{}", other.0)
    }
}

#[derive(Debug, Display, PartialEq, Error)]
#[cfg(not(tarpaulin_include))]
pub enum ServiceError {
    #[display(fmt = "internal server error")]
    InternalServerError,

    #[display(
        fmt = "This server is is closed for registration. Contact admin if this is unexpecter"
    )]
    ClosedForRegistration,

    #[display(fmt = "The value you entered for email is not an email")] //405j
    NotAnEmail,
    #[display(fmt = "The value you entered for URL is not a URL")] //405j
    NotAUrl,

    #[display(fmt = "{}", _0)]
    DBError(DBErrorWrapper),

    /// DNS challenge value is already taken
    #[display(fmt = "DNS challenge is already taken")]
    DuplicateChallengeText,

    /// DNS challenge hostname is already taken
    #[display(fmt = "DNS challenge hostname is already taken")]
    DuplicateChallengeHostname,

    /// Hostname is already taken
    #[display(fmt = "Hostname is already taken")]
    DuplicateHostname,

    /// Forge Type is already taken
    #[display(fmt = "Forge Type is already taken")]
    DuplicateForgeType,

    /// HTML link Type is already taken
    #[display(fmt = "User HTML link is already taken")]
    DuplicateUserLink,

    /// Topic is already taken
    #[display(fmt = "Topic is already taken")]
    DuplicateTopic,

    /// Repository link is already taken
    #[display(fmt = "Repository link is already taken")]
    DuplicateRepositoryLink,
}

#[derive(Serialize, Deserialize)]
#[cfg(not(tarpaulin_include))]
pub struct ErrorToResponse {
    pub error: String,
}

#[cfg(not(tarpaulin_include))]
impl ResponseError for ServiceError {
    #[cfg(not(tarpaulin_include))]
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .append_header((header::CONTENT_TYPE, "application/json; charset=UTF-8"))
            .body(
                serde_json::to_string(&ErrorToResponse {
                    error: self.to_string(),
                })
                .unwrap(),
            )
    }

    #[cfg(not(tarpaulin_include))]
    fn status_code(&self) -> StatusCode {
        match self {
            ServiceError::ClosedForRegistration => StatusCode::FORBIDDEN,
            ServiceError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            ServiceError::NotAnEmail => StatusCode::BAD_REQUEST,
            ServiceError::NotAUrl => StatusCode::BAD_REQUEST,
            ServiceError::DBError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ServiceError::DuplicateChallengeHostname
            | ServiceError::DuplicateHostname
            | ServiceError::DuplicateUserLink
            | ServiceError::DuplicateTopic
            | ServiceError::DuplicateRepositoryLink => StatusCode::BAD_REQUEST,

            ServiceError::DuplicateChallengeText | ServiceError::DuplicateForgeType => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}

impl From<DBError> for ServiceError {
    #[cfg(not(tarpaulin_include))]
    fn from(e: DBError) -> ServiceError {
        println!("from conversin: {}", e);
        ServiceError::DBError(DBErrorWrapper(e))
        //        match e {
        //            // TODO: resolve all errors to ServiceError::*
        //            _ => ServiceError::DBError(DBErrorWrapper(e)),
        //        }
    }
}

impl From<ValidationErrors> for ServiceError {
    #[cfg(not(tarpaulin_include))]
    fn from(_: ValidationErrors) -> ServiceError {
        ServiceError::NotAnEmail
    }
}

impl From<ParseError> for ServiceError {
    #[cfg(not(tarpaulin_include))]
    fn from(_: ParseError) -> ServiceError {
        ServiceError::NotAUrl
    }
}

#[cfg(not(tarpaulin_include))]
pub type ServiceResult<V> = std::result::Result<V, ServiceError>;
