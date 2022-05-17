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
//! represents all the ways a trait can fail using this crate
use std::error::Error as StdError;

//use derive_more::{error, Error as DeriveError};
use thiserror::Error;

/// Error data structure grouping various error subtypes
#[derive(Debug, Error)]
pub enum DBError {
    /// DNS challenge value is already taken
    #[error("DNS challenge is already taken")]
    DuplicateChallengeText,

    /// DNS challenge hostname is already taken
    #[error("DNS challenge hostname is already taken")]
    DuplicateChallengeHostname,

    /// Hostname is already taken
    #[error("Hostname is already taken")]
    DuplicateHostname,

    /// Forge Type is already taken
    #[error("Forge Type is already taken")]
    DuplicateForgeType,

    /// HTML link Type is already taken
    #[error("User HTML link is already taken")]
    DuplicateUserLink,

    /// User profile photo link Type is already taken
    #[error("User profile photo link is already taken")]
    DuplicateProfilePhotoLink,

    /// Topic is already taken
    #[error("Topic is already taken")]
    DuplicateTopic,

    /// Repository link is already taken
    #[error("Repository link is already taken")]
    DuplicateRepositoryLink,

    /// forge instance type is unknown
    #[error("Unknown forge instance specifier {}", _0)]
    UnknownForgeType(String),

    /// errors that are specific to a database implementation
    #[error("{0}")]
    DBError(#[source] BoxDynError),
}

/// Convenience type alias for grouping driver-specific errors
pub type BoxDynError = Box<dyn StdError + 'static + Send + Sync>;

/// Generic result data structure
pub type DBResult<V> = std::result::Result<V, DBError>;
