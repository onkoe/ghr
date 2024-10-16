//! `hash`: Creates a unique identifier for this computer.

use std::collections::HashMap;

use crate::prelude::internal::*;

use argon2::{password_hash::SaltString, Argon2, PasswordHasher as _};

pub type Hash = Vec<u8>;

/// Creates a hash from the system MAC address as the password, and various
/// other static identifiers as pieces of the salt.
pub(crate) fn make_hash() -> GhrResult<Hash> {
    let mac_addr = mac_addr()?; // the 'password' we'll hash
    let a2 = Argon2::default();

    // make the salt for hashing
    let salt = SaltString::encode_b64(&system_sources())
        .map_err(|e| GhrError::SaltFailed(e.to_string()))?;

    // try hashing pass + salt
    let hash = a2
        .hash_password(&mac_addr, &salt)
        .map_err(|e| GhrError::HashFailed(e.to_string()))?
        .hash
        .ok_or(GhrError::HashFailed(
            "Couldn't create a hash, but `argon2` gave no error.".into(),
        ))?;

    // return it as a byte array
    Ok(hash.as_bytes().to_vec())
}

/// Grabs the system's mac address, hashed and salted with a static string,
/// and trimmed to 32 bytes, which is repeated three times.
fn mac_addr() -> GhrResult<Hash> {
    // TODO: ensure this library chooses correctly
    // let mac_addrs = MacAddressIterator::new().map_err(|_| GhrError::NoMacAddresses);

    // TODO: mess with the mac addr a lil bit
    mac_address::get_mac_address()
        .map_err(|_| GhrError::NoMacAddresses)?
        .ok_or(GhrError::NoMacAddresses)
        .map(|addr| addr.bytes().to_vec())
}

/// Grabs some unique identifiers from system sources, used to make a salt.
///
/// These are truncated to avoid being identifiable.
fn system_sources() -> Hash {
    let mut sources: Vec<Hash> = Vec::new();
    const MINIMUM_REC_SALT_LEN: usize = 16_usize; // in bytes
    const MAXIMUM_SALT_LEN: usize = 64_usize; // ...also in bytes

    // try reading the smbios stuff.
    //
    // if we get anything, we'll truncate them weirdly and add them to the sources
    let smbios: Result<HashMap<&str, String>, ()> = Err(()); // TODO
    if let Ok(smbios_values) = smbios {
        // bios uuid (TODO)
        if let Some(something_value) = smbios_values.get("something") {
            sources.push(something_value.as_bytes()[..2].to_vec());
        }

        // ...
    }

    // if we still have no values, we'll add a questionable one: the SMBIOS CPU
    // serial number
    // let cpu_serial = ();

    // finally, if no other values are found, we'll just use this static bytestring D:
    const STATIC_SALT_OH_GOD: [u8; 16] = [
        0x6b, 0xf1, 0xad, 0x5f, 0x2d, 0x80, 0x8d, 0x6b, 0xb8, 0x93, 0xa9, 0xa3, 0x62, 0x1e, 0x67,
        0x1a,
    ];

    if let Some(slice) =
        STATIC_SALT_OH_GOD.get(..(MINIMUM_REC_SALT_LEN.saturating_sub(sources.len())))
    {
        sources.push(slice.to_vec());
    }

    // now, we'll flatten all the data sources and collect into a single vec
    let mut flattened = sources.into_iter().flatten().collect::<Vec<u8>>();

    // check that length is as expected, then return within range
    debug_assert!(
        flattened.len() >= MINIMUM_REC_SALT_LEN,
        "the total byte count should always be over {MINIMUM_REC_SALT_LEN}, but it's actually {}",
        flattened.len()
    );

    let max = MAXIMUM_SALT_LEN.min(flattened.len()); // don't go over 64 bytes
    flattened.drain(..max).collect::<Vec<u8>>()
}

#[cfg(test)]
mod tests {
    use super::mac_addr;

    #[test]
    fn finds_mac_addr() {
        // note: even CI virtual machines should have a MAC addr
        assert!(mac_addr().is_ok());
    }
}
