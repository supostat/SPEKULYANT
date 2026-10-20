//! Детерминированный случай мира: один сид — одна история.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::SeedableRng;

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
/// Старшие биты хэша, из которых складывается доля: столько их у мантиссы f64.
const FRACTION_BITS: u32 = 53;

/// Генератор мира на ChaCha8 с именованными подпотоками: у каждой подсистемы свой поток
/// (`stream("prices")`, `stream("events")`), поэтому новые розыгрыши в одной подсистеме
/// не сдвигают последовательности других.
#[derive(Clone, Debug, Serialize, Deserialize)]
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

/// Номер потока ChaCha по имени, одинаковый на всех платформах и версиях Rust.
fn stream_number(name: &str) -> u64 {
    fnv1a(name.bytes())
}

/// Равномерная доля от нуля до единицы, не включая её, из старших битов хэша: случай без потока,
/// который при тех же байтах выпадает так же.
pub(crate) fn fraction_of(hash: u64) -> f64 {
    (hash >> (u64::BITS - FRACTION_BITS)) as f64 / (1_u64 << FRACTION_BITS) as f64
}

/// Перемешивание SplitMix64: соседние числа дают несвязанные хэши.
pub(crate) fn mix(value: u64) -> u64 {
    let value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    let value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

/// FNV-1a 64: устойчивый хэш байтов — номера потоков и отпечаток данных игры.
pub(crate) fn fnv1a(bytes: impl IntoIterator<Item = u8>) -> u64 {
    bytes.into_iter().fold(FNV_OFFSET_BASIS, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME)
    })
}
