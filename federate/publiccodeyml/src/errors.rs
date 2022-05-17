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

use serde_yaml::Error as YamlError;
use thiserror::Error;
use tokio::io::Error as IOError;

use db_core::errors::DBError;

/// Error data structure grouping various error subtypes
#[derive(Debug, Error)]
pub enum FederateErorr {
    /// serialization error
    #[error("Serialization error: {0}")]
    SerializationError(YamlError),
    /// database errors
    #[error("{0}")]
    DBError(DBError),

    /// IO Error
    #[error("{0}")]
    IOError(IOError),
}

impl From<DBError> for FederateErorr {
    fn from(e: DBError) -> Self {
        Self::DBError(e)
    }
}

impl From<IOError> for FederateErorr {
    fn from(e: IOError) -> Self {
        Self::IOError(e)
    }
}

impl From<YamlError> for FederateErorr {
    fn from(e: YamlError) -> Self {
        Self::SerializationError(e)
    }
}

/// Convenience type alias for grouping driver-specific errors
pub type BoxDynError = Box<dyn StdError + 'static + Send + Sync>;

/// Generic result data structure
pub type FResult<V> = std::result::Result<V, FederateErorr>;
