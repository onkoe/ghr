//! `system_config`: Info about the system's configuration

use sleep::Sleep;

use crate::prelude::internal::*;

pub mod sleep;

#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, TypeScript)]
#[non_exhaustive]
pub struct SystemConfInfo {
    pub sleep: Sleep,
}
