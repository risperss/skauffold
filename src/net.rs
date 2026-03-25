use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::HashMap;

use crate::util::{State, concat_bits, get_random_func};

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
        // They must meet inside the cycle after at most mu + lambda iterations
        // (where mu = transient, lambda = cycle length).
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

        // ── Phase 2: find cycle length lambda ────────────────────────────────
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

        // ── Phase 3: find transient mu ────────────────────────────────────────
        // Reset one pointer to the initial state, keep the other at the meeting
        // point (which is on the cycle, exactly lamdba steps from itself).
        // Advance both one step at a time; they meet at the cycle entry point
        // after exactly mu steps.
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

    fn perform_run_hashset(&mut self, rng: &mut impl Rng) -> RunResult {
        let mut seen: HashMap<State, usize> = HashMap::new();

        let mut state = State::new(self.n);
        Self::set_random_state(self.n, &mut state, rng);

        for t in 0..self.max_steps {
            if let Some(&prev_t) = seen.get(&state) {
                let mut states: Vec<State> = seen.keys().cloned().collect();
                states.sort_by_key(|t| seen[t]);
                return RunResult {
                    transient: prev_t,
                    cycle_length: t - prev_t,
                    max_steps_reached: false,
                    cycle_id: Some(states[prev_t..].iter().min().cloned().unwrap()),
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
fn net_run_hashset_terminates_and_is_consistent() {
    let mut net = Net::new(100, 2, false, 10_000, 42);
    let mut rng = StdRng::seed_from_u64(42);
    let result = net.perform_run(&mut rng, false);

    assert!(!result.max_steps_reached);
    assert!(result.cycle_length > 0);
    assert!(result.cycle_id.is_some());
}

#[test]
fn net_run_floyd_terminates_and_is_consistent() {
    let mut net = Net::new(10, 2, false, 10_000, 42);
    let mut rng = StdRng::seed_from_u64(42);
    let result = net.perform_run(&mut rng, true);

    assert!(!result.max_steps_reached);
    assert!(result.cycle_length > 0);
    assert!(result.cycle_id.is_some());
}

#[test]
fn net_floyd_and_hashset_agree_across_seeds() {
    for seed in [0u64, 1, 42, 537, 9999] {
        let mut net = Net::new(512, 2, false, 10_000, seed);
        let mut rng_floyd = StdRng::seed_from_u64(seed);
        let mut rng_hashset = StdRng::seed_from_u64(seed);

        let floyd = net.perform_run(&mut rng_floyd, true);
        let hashset = net.perform_run(&mut rng_hashset, false);

        assert_eq!(
            floyd.cycle_id, hashset.cycle_id,
            "seed={seed}: cycle_id mismatch"
        );
        assert_eq!(
            floyd.cycle_length, hashset.cycle_length,
            "seed={seed}: cycle_length mismatch"
        );
        assert_eq!(
            floyd.max_steps_reached, hashset.max_steps_reached,
            "seed={seed}: max_steps_reached mismatch"
        );
    }
}

#[test]
fn net_max_steps_reached_when_limit_is_tiny() {
    // A limit of 2 steps can't find any cycle in a non-trivial net
    let mut net = Net::new(20, 3, false, 2, 522);
    let mut rng = StdRng::seed_from_u64(42);
    let result = net.perform_run(&mut rng, true);
    assert!(result.max_steps_reached);
    assert!(result.cycle_id.is_none());
}

#[test]
fn net_cycle_id_is_deterministic_for_same_network_and_initial_state() {
    // Same net seed + same run seed -> identical RunResult both times.
    let mut net = Net::new(12, 2, false, 10_000, 522);
    let mut rng1 = StdRng::seed_from_u64(77);
    let mut rng2 = StdRng::seed_from_u64(77);

    let r1 = net.perform_run(&mut rng1, true);
    let r2 = net.perform_run(&mut rng2, true);

    assert_eq!(r1.cycle_id, r2.cycle_id);
    assert_eq!(r1.cycle_length, r2.cycle_length);
    assert_eq!(r1.transient, r2.transient);
}
