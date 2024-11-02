//! `system_config`: Info about the system's configuration

pub mod sleep;

use crate::prelude::internal::*;
use sleep::Sleep;

use futures::FutureExt as _;

/// Information about the system configuration and standard support.
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, TypeScript)]
#[non_exhaustive]
pub struct SystemConfInfo {
    pub sleep: Sleep,
}

impl SystemConfInfo {
    pub async fn new() -> Self {
        // we'll get all info using heap-allocated futures.
        //
        // this avoids overflowing the stack.
        let sleep = sleep::get().boxed().await;

        SystemConfInfo { sleep }
    }
}
