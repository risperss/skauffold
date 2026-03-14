use bitvec::prelude::*;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::Serialize;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

// ---------------------------------------------------------------------------
// Bit-packed state vector
// ---------------------------------------------------------------------------

#[derive(Clone, PartialEq, Eq)]
struct State(BitVec);

impl Hash for State {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.0.len().hash(h);
        self.0.as_raw_slice().hash(h);
    }
}

impl State {
    fn new(len: usize) -> Self {
        State(bitvec![0; len])
    }

    fn set(&mut self, i: usize, val: bool) {
        self.0.set(i, val);
    }

    fn get(&self, i: usize) -> bool {
        self.0[i]
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn concat_bits(state: &State, inputs: &[usize]) -> u64 {
    inputs
        .iter()
        .fold(0u64, |acc, &i| (acc << 1) | state.get(i) as u64)
}

fn get_random_func(k: u32, exclude_taut_and_cont: bool, rng: &mut impl Rng) -> u64 {
    let max = (1u64 << (1u64 << k)) - 1;
    if exclude_taut_and_cont {
        rng.gen_range(1..max)
    } else {
        rng.gen_range(0..=max)
    }
}

// ---------------------------------------------------------------------------
// Network
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct RunResult {
    pub transient: usize,
    pub cycle_length: usize,
    pub max_steps_reached: bool,
    pub activities: Vec<u16>,
}

pub struct Net {
    n: usize,
    k: usize,
    funcs: Vec<u64>,
    inputs: Vec<usize>,
    current: State,
    next: State,
    max_steps: usize,
    seen: HashMap<State, usize>,
    activities: Vec<u16>,
}

impl Net {
    pub fn new(n: usize, k: u32, exclude_taut_and_cont: bool, max_steps: usize, seed: u64) -> Self {
        assert!(k <= 6, "k must be <= 6");
        let mut rng = StdRng::seed_from_u64(seed);

        let funcs: Vec<u64> = (0..n)
            .map(|_| get_random_func(k, exclude_taut_and_cont, &mut rng))
            .collect();

        let inputs: Vec<usize> = (0..n * k as usize).map(|_| rng.gen_range(0..n)).collect();

        Net {
            n,
            k: k as usize,
            funcs,
            inputs,
            current: State::new(n),
            next: State::new(n),
            max_steps,
            seen: HashMap::with_capacity(max_steps),
            activities: Vec::with_capacity(max_steps),
        }
    }

    fn set_random_state(&mut self, rng: &mut impl Rng) {
        for i in 0..self.n {
            self.current.set(i, rng.r#gen());
        }
    }

    fn step(&mut self) {
        // Number of nodes which change value in a given state transition
        let mut activity: u16 = 0;

        for i in 0..self.n {
            let input_start_idx = i * self.k;
            let idx = concat_bits(
                &self.current,
                &self.inputs[input_start_idx..input_start_idx + self.k],
            );

            let prev = self.current.get(i);
            let next = (self.funcs[i] >> idx) & 1 != 0;
            if prev != next {
                activity += 1;
            }

            self.next.set(i, next);
        }

        self.activities.push(activity);
        std::mem::swap(&mut self.current, &mut self.next);
    }

    pub fn perform_run(&mut self, rng: &mut impl Rng) -> RunResult {
        self.set_random_state(rng);
        self.seen.clear();
        self.activities.clear();

        for t in 0..self.max_steps {
            let state = self.current.clone();
            if let Some(&prev_t) = self.seen.get(&state) {
                return RunResult {
                    transient: prev_t,
                    cycle_length: t - prev_t,
                    max_steps_reached: false,
                    activities: self.activities.clone(),
                };
            }
            self.seen.insert(state, t);
            self.step();
        }

        RunResult {
            transient: self.max_steps,
            cycle_length: self.max_steps,
            max_steps_reached: true,
            activities: self.activities.clone(),
        }
    }
}
