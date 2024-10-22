use core::error::Error;
use pisserror::Error;

pub type GhrResult<T> = Result<T, GhrError>;

const ASK_USER_TO_REPORT: &str = "This is a bug. Please report it!";

/// An error that occurred when grabbing system/hardware information.
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, Error)]
#[non_exhaustive]
pub enum GhrError {
    // function errors...
    #[error("Failed to get OS info on this system. (err: `{_0}`)")]
    OsInfoInaccessible(String),
    #[error("To create a system report with a system identifier, you must have a non-loopback MAC address.")]
    NoMacAddresses,

    #[error("Couldn't access hardware device information. (err: {_0})")]
    ComponentInfoInaccessible(String),
    #[error("Component info contained unexpected information. (err: {_0})")]
    ComponentInfoWeirdInfo(String),
    #[error("Specific component unsupported under the provided configuration. (err: {_0})")]
    ComponentUnsupported(String),
    #[error("Failed to create regular rexpression. (err: {_0}")]
    RegexCreationFailure(String),

    // errors related to generally-unexpected failures
    #[error("Couldn't create a salt for the system identification hash. {ASK_USER_TO_REPORT} (err: {_0})")]
    SaltFailed(String),
    #[error(
        "Failed to hash this device's MAC address for the system identification. (error: {_0})"
    )]
    HashFailed(String),
}
