//! A list of utilities for Eludris.

use rand::{thread_rng, Rng};
use std::time::{SystemTime, UNIX_EPOCH};

#[allow(dead_code)]
const BIGINT_LIMIT: u32 = 4294967295; // Biggest 32-bit number.

/// A function that returns the current UNIX timestamp with eludris' custom epoch.
pub fn now_timestamp() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Couldn't determine unix timestamp")
        .as_secs() as u32
        - 1_650_000_000
}

/// A function that generates an id based on a provided sequence number.
#[allow(dead_code)]
pub fn generate_id(sequence: u16) -> u64 {
    let now = now_timestamp();
    let timestamp = now % BIGINT_LIMIT;
    let overflow = now / BIGINT_LIMIT;
    (timestamp as u64) << 32
        | (sequence as u64) << 16
        | (thread_rng().gen_range::<u16, _>(0..4096) as u64) << 4
        | overflow as u64
}

#[cfg(test)]
mod tests {
    use super::generate_id;
    use rand::{thread_rng, Rng};

    #[test]
    fn id_sequence() {
        let sequence: u16 = thread_rng().gen_range::<u16, _>(0..4096);
        let id = generate_id(sequence);
        assert_eq!(((id >> 16) & 0xFFFF) as u16, sequence);
    }
}
