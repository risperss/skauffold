use bitvec::prelude::*;
use rand::Rng;
use std::hash::{Hash, Hasher};

// ---------------------------------------------------------------------------
// Bit-packed state vector
// ---------------------------------------------------------------------------

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct State(BitVec);

impl Hash for State {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.0.len().hash(h);
        self.0.as_raw_slice().hash(h);
    }
}

impl State {
    pub fn new(len: usize) -> Self {
        State(bitvec![0; len])
    }

    pub fn set(&mut self, i: usize, val: bool) {
        self.0.set(i, val);
    }

    pub fn get(&self, i: u16) -> bool {
        self.0[i as usize]
    }

    pub fn hamming_distance(&self, other: &State) -> usize {
        self.0
            .iter()
            .zip(other.0.iter())
            .filter(|(a, b)| a != b)
            .count()
    }
}

pub fn concat_bits(state: &State, inputs: &[u16]) -> u64 {
    inputs
        .iter()
        .fold(0u64, |acc, &i| (acc << 1) | state.get(i) as u64)
}

pub fn get_random_func(k: u32, exclude_taut_and_cont: bool, rng: &mut impl Rng) -> u64 {
    let max = (1u64 << (1u64 << k)) - 1;
    if exclude_taut_and_cont {
        rng.gen_range(1..max)
    } else {
        rng.gen_range(0..=max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    fn make_state(n: usize, bits: &[(usize, bool)]) -> State {
        let mut state = State::new(n);
        for &(idx, val) in bits {
            state.set(idx, val);
        }
        state
    }

    // ---------------------------------------------------------------------------
    // concat_bits
    // ---------------------------------------------------------------------------

    #[test]
    fn concat_bits_empty_inputs_returns_zero() {
        let state = State::new(4);
        assert_eq!(concat_bits(&state, &[]), 0u64);
    }

    #[test]
    fn concat_bits_single_one() {
        let state = make_state(4, &[(0, true)]);
        assert_eq!(concat_bits(&state, &[0]), 1u64);
    }

    #[test]
    fn concat_bits_multiple_bits() {
        // bits in order: 1, 0, 1 -> 0b101 = 5
        let state = make_state(4, &[(0, true), (1, false), (2, true)]);
        assert_eq!(concat_bits(&state, &[0, 1, 2]), 0b101u64);
    }

    #[test]
    fn concat_bits_respects_input_order() {
        // index 0 = true, index 1 = false
        // [0, 1] -> 1,0 -> 0b10 = 2
        // [1, 0] -> 0,1 -> 0b01 = 1
        let state = make_state(4, &[(0, true), (1, false)]);
        assert_eq!(concat_bits(&state, &[0, 1]), 0b10u64);
        assert_eq!(concat_bits(&state, &[1, 0]), 0b01u64);
    }

    // ---------------------------------------------------------------------------
    // get_random_func
    // ---------------------------------------------------------------------------

    #[test]
    fn get_random_func_exclude_taut_and_cont_never_zero_or_max() {
        let mut rng = StdRng::seed_from_u64(99);
        for k in 1u32..=4 {
            let max = (1u64 << (1u64 << k)) - 1;
            for _ in 0..500 {
                let v = get_random_func(k, true, &mut rng);
                assert!(v > 0, "k={k}: got contradiction (0)");
                assert!(v < max, "k={k}: got tautology (max={max})");
            }
        }
    }

    #[test]
    fn get_random_func_include_taut_and_cont_stays_in_range() {
        let mut rng = StdRng::seed_from_u64(7);
        for k in 1u32..=4 {
            let max = (1u64 << (1u64 << k)) - 1;
            for _ in 0..500 {
                let v = get_random_func(k, false, &mut rng);
                assert!(v <= max, "k={k}: value {v} exceeds max {max}");
            }
        }
    }

    #[test]
    fn get_random_func_k1_can_produce_full_range() {
        // k=1: 2^(2^1) = 4 possible functions, values 0..=3
        let mut rng = StdRng::seed_from_u64(42);
        let results: std::collections::HashSet<u64> = (0..2000)
            .map(|_| get_random_func(1, false, &mut rng))
            .collect();
        assert!(results.contains(&0), "never produced contradiction");
        assert!(results.contains(&3), "never produced tautology");
    }

    #[test]
    fn get_random_func_deterministic_with_seed() {
        let mut rng1 = StdRng::seed_from_u64(123);
        let mut rng2 = StdRng::seed_from_u64(123);
        for _ in 0..20 {
            assert_eq!(
                get_random_func(3, true, &mut rng1),
                get_random_func(3, true, &mut rng2)
            );
        }
    }
}
