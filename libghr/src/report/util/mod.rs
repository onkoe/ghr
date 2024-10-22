#[cfg(target_os = "linux")]
pub(crate) mod linux;

pub(crate) fn unit_to_mibiunits(value: impl Into<u64>) -> u32 {
    const UNIT_CONV: u64 = 1_048_576;

    let value = Into::<u64>::into(value);
    (value / UNIT_CONV) as u32
}
