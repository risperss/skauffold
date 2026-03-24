use bitvec::prelude::*;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::HashMap;
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
    fn new(len: usize) -> Self {
        State(bitvec![0; len])
    }

    fn set(&mut self, i: usize, val: bool) {
        self.0.set(i, val);
    }

    fn get(&self, i: u16) -> bool {
        self.0[i as usize]
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn concat_bits(state: &State, inputs: &[u16]) -> u64 {
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
#[derive(Debug, Eq, PartialEq)]
pub struct RunResult {
    pub transient: usize,
    pub cycle_length: usize,
    pub max_steps_reached: bool,
    /// The lexicographic minimum state on the cycle, used as a stable cycle ID.
    /// `None` iff `max_steps_reached`.
    pub cycle_id: Option<State>,
    pub states: Option<Vec<State>>,
}

pub struct Net {
    n: usize,
    k: usize,
    funcs: Vec<u64>,
    inputs: Vec<u16>,
    /// Scratch buffer – used during `step` to avoid allocation.
    buff: State,
    max_steps: usize,
}

impl Net {
    pub fn new(n: usize, k: u32, exclude_taut_and_cont: bool, max_steps: usize, seed: u64) -> Self {
        assert!(k <= 6, "k must be <= 6");
        let mut rng = StdRng::seed_from_u64(seed);

        let funcs: Vec<u64> = (0..n)
            .map(|_| get_random_func(k, exclude_taut_and_cont, &mut rng))
            .collect();

        let inputs: Vec<u16> = (0..n * k as usize)
            .map(|_| rng.gen_range(0..n) as u16)
            .collect();

        Net {
            n,
            k: k as usize,
            funcs,
            inputs,
            buff: State::new(n),
            max_steps,
        }
    }

    // Advance `state` by one step, writing the result back into `state`.
    // Uses `self.next` as a scratch buffer to avoid allocation.
    fn step_state(&mut self, state: &mut State) {
        for i in 0..self.n {
            let start = i * self.k;
            let idx = concat_bits(state, &self.inputs[start..start + self.k]);
            self.buff.set(i, (self.funcs[i] >> idx) & 1 != 0);
        }
        std::mem::swap(state, &mut self.buff);
    }

    // Advance `state` by `steps` steps in-place.
    fn advance(&mut self, state: &mut State, steps: usize) {
        for _ in 0..steps {
            self.step_state(state);
        }
    }

    fn set_random_state(n: usize, state: &mut State, rng: &mut impl Rng) {
        for i in 0..n {
            state.set(i, rng.r#gen());
        }
    }

    pub fn perform_run(&mut self, rng: &mut impl Rng, use_floyd: bool) -> RunResult {
        if use_floyd {
            self.perform_run_floyd(rng)
        } else {
            self.perform_run_hashset(rng)
        }
    }

    /// Floyd's tortoise-and-hare cycle detection.
    ///
    /// Memory usage: O(N) — only a fixed number of `State` clones are live at
    /// any one time (tortoise, hare, initial, and a few temporaries), regardless
    /// of cycle length or transient length.
    fn perform_run_floyd(&mut self, rng: &mut impl Rng) -> RunResult {
        // ── record the initial state so we can replay from it ────────────────
        let mut initial = State::new(self.n);
        Self::set_random_state(self.n, &mut initial, rng);

        // ── Phase 1: detect *a* meeting point ────────────────────────────────
        // Tortoise takes 1 step, hare takes 2, per iteration.
        // They must meet inside the cycle after at most μ + λ iterations
        // (where μ = transient, λ = cycle length).
        let mut tortoise = initial.clone();
        let mut hare = initial.clone();

        let mut phase1_steps = 0usize;
        loop {
            if phase1_steps >= self.max_steps {
                return RunResult {
                    transient: 0,
                    cycle_length: 0,
                    max_steps_reached: true,
                    cycle_id: None,
                    states: None,
                };
            }
            self.advance(&mut tortoise, 1);
            self.advance(&mut hare, 2);
            phase1_steps += 1;
            if tortoise == hare {
                break;
            }
        }
        // `phase1_steps` == number of single-steps tortoise has taken.
        // Both pointers are now somewhere on the cycle.

        // ── Phase 2: find cycle length λ ─────────────────────────────────────
        // Keep tortoise fixed; advance hare one step at a time until it laps
        // back to tortoise.
        let mut cycle_length = 0usize;
        // hare starts one step ahead so we enter the loop body at least once.
        self.advance(&mut hare, 1);
        cycle_length += 1;
        while tortoise != hare {
            self.advance(&mut hare, 1);
            cycle_length += 1;
        }

        // ── Phase 3: find transient μ ─────────────────────────────────────────
        // Reset one pointer to the initial state, keep the other at the meeting
        // point (which is on the cycle, exactly λ steps from itself).
        // Advance both one step at a time; they meet at the cycle entry point
        // after exactly μ steps.
        let mut p1 = initial.clone();
        let mut p2 = initial.clone();
        self.advance(&mut p2, phase1_steps); // p2 is back at the meeting point

        let mut transient = 0usize;
        while p1 != p2 {
            self.advance(&mut p1, 1);
            self.advance(&mut p2, 1);
            transient += 1;
        }
        // p1 (== p2) is now the cycle entry state.

        // ── Phase 4: find cycle_id (min state on the cycle) ──────────────────
        let cycle_entry = p1; // rename for clarity
        let mut min_state = cycle_entry.clone();
        let mut cursor = cycle_entry.clone();
        for _ in 1..cycle_length {
            self.advance(&mut cursor, 1);
            if cursor < min_state {
                min_state = cursor.clone();
            }
        }

        RunResult {
            transient,
            cycle_length,
            max_steps_reached: false,
            cycle_id: Some(min_state),
            states: None,
        }
    }

    // Use a hashset to record seen states
    fn perform_run_hashset(&mut self, rng: &mut impl Rng) -> RunResult {
        let mut seen: HashMap<State, usize> = HashMap::new();

        let mut state = State::new(self.n);
        Self::set_random_state(self.n, &mut state, rng);

        for t in 0..self.max_steps {
            if let Some(&prev_t) = seen.get(&state) {
                let mut states: Vec<State> = seen.keys().cloned().collect();
                states.sort_by_key(|t| seen[t]);
                println!("{:?}", states[prev_t + 1..].iter());
                return RunResult {
                    transient: prev_t,
                    cycle_length: t - prev_t,
                    max_steps_reached: false,
                    cycle_id: Some(states[prev_t + 1..].iter().min().cloned().unwrap()),
                    states: Some(states),
                };
            }
            seen.insert(state.clone(), t);
            self.advance(&mut state, 1);
        }

        RunResult {
            transient: 0,
            cycle_length: 0,
            max_steps_reached: true,
            cycle_id: None,
            states: None,
        }
    }
}

#[test]
fn test_floyd_vs_hashset() {
    let mut net = Net::new(10, 2, false, 10000, 537);

    let mut run_rng_floyd = StdRng::seed_from_u64(537);
    let mut run_rng_hashset = StdRng::seed_from_u64(537);

    let result_floyd = net.perform_run(&mut run_rng_floyd, true);
    let result_hashset = net.perform_run(&mut run_rng_hashset, false);

    assert_eq!(
        (
            result_floyd.cycle_id,
            result_floyd.cycle_length,
            result_floyd.transient
        ),
        (
            result_hashset.cycle_id,
            result_hashset.cycle_length,
            result_floyd.transient
        )
    );
}
