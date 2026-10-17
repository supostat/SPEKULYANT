//! Детерминированный случай мира: один сид — одна история.

use std::collections::BTreeMap;

use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::SeedableRng;

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Генератор мира на ChaCha8 с именованными подпотоками: у каждой подсистемы свой поток
/// (`stream("prices")`, `stream("events")`), поэтому новые розыгрыши в одной подсистеме
/// не сдвигают последовательности других.
#[derive(Clone)]
pub struct WorldRng {
    seed: u64,
    streams: BTreeMap<String, ChaCha8Rng>,
}

impl WorldRng {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            streams: BTreeMap::new(),
        }
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn stream(&mut self, name: &str) -> &mut ChaCha8Rng {
        let seed = self.seed;
        self.streams.entry(name.to_owned()).or_insert_with(|| {
            let mut stream = ChaCha8Rng::seed_from_u64(seed);
            stream.set_stream(stream_number(name));
            stream
        })
    }
}

/// FNV-1a 64 от имени: номер потока ChaCha, одинаковый на всех платформах и версиях Rust.
fn stream_number(name: &str) -> u64 {
    name.bytes().fold(FNV_OFFSET_BASIS, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME)
    })
}
