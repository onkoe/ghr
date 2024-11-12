#[cfg(any(target_os = "linux", target_os = "android"))]
pub(crate) mod linux;

#[tracing::instrument(skip(value))]
/// converts a unit from units to mibiunits. for example, 1024^2 bytes
/// becomes 1 MiB.
pub(crate) fn unit_to_mibiunits(value: impl Into<u64>) -> u32 {
    const UNIT_CONV: u64 = 1_048_576;

    let value = Into::<u64>::into(value);
    (value / UNIT_CONV) as u32
}

/// this helper function assists in creating a logger in unit tests.
///
/// it's exported in the `internal` prelude when `cfg(test)` is true.
#[cfg(test)]
#[tracing::instrument]
pub(crate) fn logger() {
    _ = tracing_subscriber::fmt()
        .pretty()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();
}

#[cfg(test)]
mod tests {
    use crate::report::unit_to_mibiunits;

    #[test]
    fn one_gib_to_mib() {
        let one_gibibyte = 1_048_576_u64 * 1024; // in bytes
        assert_eq!(unit_to_mibiunits(one_gibibyte), 1024_u32);
    }

    #[test]
    fn _1000_bytes_is_zero_mibs() {
        let megabyte = 1000_u64 * 1000;
        assert_eq!(unit_to_mibiunits(megabyte), 0_u32);
    }

    #[test]
    fn _1024_bytes_is_one_mib() {
        let mibibyte = 1024_u64 * 1024;
        assert_eq!(unit_to_mibiunits(mibibyte), 1_u32);
    }
}
