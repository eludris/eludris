//! A simple collection of ID related utilities.

use std::time::{Duration, SystemTime};

lazy_static! {
    pub static ref DERAILED_EPOCH: SystemTime =
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_650_000_000);
}

/// Generate an instance id
pub fn generate_instance_id() -> u64 {
    // This is just a 48 bit Unix timestamp
    SystemTime::now()
        .duration_since(*DERAILED_EPOCH)
        .expect("Couldn't get current timestamp")
        .as_secs()
        & 0xFFFFFFFFFFFF
}

/// An abstraction for generating spec-compliant IDs and handling incrementing them
///
/// ## Example
///
/// ```rust
/// use todel::ids::{IDGenerator, generate_instance_id};
///
/// let instance_id = generate_instance_id(); // This is ideally fetched from a database.
/// let mut generator = IDGenerator::new(instance_id); // Create a new ID generator with your instance ID.
///
/// generator.generate_id(); // Generate an ID which also increments the sequence.
/// ```
#[derive(Debug, Clone)]
pub struct IDGenerator {
    instance_id: u64,
    sequence: u16,
}

impl IDGenerator {
    /// Create a new IDGenerator from an instance ID.
    pub fn new(instance_id: u64) -> Self {
        Self {
            instance_id,
            sequence: 0,
        }
    }

    /// Generate a new ID and handle incrementing the sequence
    pub fn generate_id(&mut self) -> u128 {
        if self.sequence == u16::MAX {
            self.sequence = 0
        } else {
            self.sequence += 1;
        }
        (SystemTime::now()
            .duration_since(*DERAILED_EPOCH)
            .expect("Couldn't get current timestamp")
            .as_secs() as u128)
            << 64
            | (self.instance_id as u128) << 16
            | self.sequence as u128
    }
}

#[cfg(test)]
mod tests {
    use super::{generate_instance_id, IDGenerator};

    #[test]
    fn id_generator() {
        let instance_id = generate_instance_id();
        let mut generator = IDGenerator::new(instance_id);

        let id = generator.generate_id();
        assert_eq!(id & 0xFFFF, 1);
        assert_eq!((id & 0xFFFFFFFFFFFF0000) >> 16, instance_id as u128);

        let id = generator.generate_id();
        assert_eq!(id & 0xFFFF, 2);
        assert_eq!((id & 0xFFFFFFFFFFFF0000) >> 16, instance_id as u128);
    }

    #[test]
    fn id_generator_overflow() {
        let instance_id = generate_instance_id();
        let mut generator = IDGenerator {
            instance_id,
            sequence: u16::MAX - 1,
        };

        let id = generator.generate_id();
        assert_eq!(id & 0xFFFF, u16::MAX as u128);
        assert_eq!((id & 0xFFFFFFFFFFFF0000) >> 16, instance_id as u128);

        let id = generator.generate_id();
        assert_eq!(id & 0xFFFF, 0);
        assert_eq!((id & 0xFFFFFFFFFFFF0000) >> 16, instance_id as u128);

        let id = generator.generate_id();
        assert_eq!(id & 0xFFFF, 1);
        assert_eq!((id & 0xFFFFFFFFFFFF0000) >> 16, instance_id as u128);
    }
}
