use crate::prelude::internal::*;

/// gets info about the computer's sleep states.
#[tracing::instrument]
pub(super) async fn get() -> Sleep {
    tracing::info!("Sleep info is unimplemented on Windows. No info will be returned.");
    Sleep::default()
}
