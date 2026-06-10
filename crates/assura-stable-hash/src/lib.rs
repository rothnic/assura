#![no_std]

//! Stable non-cryptographic hashing shared by Assura cache and artifact code.
//!
//! This crate intentionally uses FNV-1a for deterministic local cache keys and
//! version fingerprints. It is not a security boundary.

use core::hash::Hasher;

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

/// Return a stable FNV-1a hash for bytes.
pub fn stable_hash(bytes: &[u8]) -> u64 {
    stable_hash_const(bytes)
}

/// Return a stable FNV-1a hash in const contexts.
pub const fn stable_hash_const(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    let mut index = 0;
    while index < bytes.len() {
        hash ^= bytes[index] as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
        index += 1;
    }
    hash
}

/// Hasher wrapper for code that needs the standard `Hasher` trait.
#[derive(Default)]
pub struct StableHasher(u64);

impl Hasher for StableHasher {
    fn write(&mut self, bytes: &[u8]) {
        self.0 = stable_hash_with_seed(self.0, bytes);
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

fn stable_hash_with_seed(seed: u64, bytes: &[u8]) -> u64 {
    let mut hash = if seed == 0 { FNV_OFFSET_BASIS } else { seed };
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}
