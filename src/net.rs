use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::HashMap;

use crate::NKInterpretation;
use crate::util::{State, concat_bits, generate_inputs, get_random_func};

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
    pub fn new(
        n: usize,
        k: u32,
        nk_interpretation: NKInterpretation,
        exclude_taut_and_cont: bool,
        max_steps: usize,
        seed: u64,
    ) -> Self {
        assert!(k <= 6, "k must be <= 6");
        let mut rng = StdRng::seed_from_u64(seed);

        let funcs: Vec<u64> = (0..n)
            .map(|_| get_random_func(k, exclude_taut_and_cont, &mut rng))
            .collect();

        let inputs: Vec<u16> = generate_inputs(n, k, nk_interpretation, &mut rng);

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

    pub fn perform_run(
        &mut self,
        initial: Option<State>,
        rng: Option<&mut impl Rng>,
        use_floyd: bool,
    ) -> RunResult {
        assert_ne!(
            initial.is_none(),
            rng.is_none(),
            "must pass in either an initial state, or an rng, but not neither or both"
        );

        if use_floyd {
            self.perform_run_floyd(initial, rng)
        } else {
            self.perform_run_hashset(initial, rng)
        }
    }

    /// Floyd's tortoise-and-hare cycle detection.
    ///
    /// Memory usage: O(N) — only a fixed number of `State` clones are live at
    /// any one time (tortoise, hare, initial, and a few temporaries), regardless
    /// of cycle length or transient length.
    fn perform_run_floyd(
        &mut self,
        initial_state: Option<State>,
        rng: Option<&mut impl Rng>,
    ) -> RunResult {
        // ── record the initial state so we can replay from it ────────────────
        let mut initial = State::new(self.n);

        if let Some(rng) = rng {
            Self::set_random_state(self.n, &mut initial, rng);
        } else {
            initial = initial_state.unwrap();
        }

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

    fn perform_run_hashset(
        &mut self,
        initial_state: Option<State>,
        rng: Option<&mut impl Rng>,
    ) -> RunResult {
        let mut seen: HashMap<State, usize> = HashMap::new();

        let mut state = State::new(self.n);

        if let Some(rng) = rng {
            Self::set_random_state(self.n, &mut state, rng);
        } else {
            state = initial_state.unwrap();
        }

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
    let mut net = Net::new(100, 2, NKInterpretation::AllN, false, 10_000, 42);
    let mut rng = StdRng::seed_from_u64(42);
    let result = net.perform_run(None, Some(&mut rng), false);

    assert!(!result.max_steps_reached);
    assert!(result.cycle_length > 0);
    assert!(result.cycle_id.is_some());
}

#[test]
fn net_run_floyd_terminates_and_is_consistent() {
    let mut net = Net::new(10, 2, NKInterpretation::AllN, false, 10_000, 42);
    let mut rng = StdRng::seed_from_u64(42);
    let result = net.perform_run(None, Some(&mut rng), true);

    assert!(!result.max_steps_reached);
    assert!(result.cycle_length > 0);
    assert!(result.cycle_id.is_some());
}

#[test]
fn net_floyd_and_hashset_agree_across_seeds() {
    for seed in [0u64, 1, 42, 537, 9999] {
        let mut net = Net::new(512, 2, NKInterpretation::AllN, false, 10_000, seed);
        let mut rng_floyd = StdRng::seed_from_u64(seed);
        let mut rng_hashset = StdRng::seed_from_u64(seed);

        let floyd = net.perform_run(None, Some(&mut rng_floyd), true);
        let hashset = net.perform_run(None, Some(&mut rng_hashset), false);

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
    let mut net = Net::new(20, 3, NKInterpretation::AllN, false, 2, 522);
    let mut rng = StdRng::seed_from_u64(42);
    let result = net.perform_run(None, Some(&mut rng), true);
    assert!(result.max_steps_reached);
    assert!(result.cycle_id.is_none());
}

#[test]
fn net_cycle_id_is_deterministic_for_same_network_and_initial_state() {
    // Same net seed + same run seed -> identical RunResult both times.
    let mut net = Net::new(12, 2, NKInterpretation::AllN, false, 10_000, 522);
    let mut rng1 = StdRng::seed_from_u64(77);
    let mut rng2 = StdRng::seed_from_u64(77);

    let r1 = net.perform_run(None, Some(&mut rng1), true);
    let r2 = net.perform_run(None, Some(&mut rng2), true);

    assert_eq!(r1.cycle_id, r2.cycle_id);
    assert_eq!(r1.cycle_length, r2.cycle_length);
    assert_eq!(r1.transient, r2.transient);
}

#[test]
fn net_initial_state_is_respected_floyd() {
    // Build two distinct initial states and confirm that passing each one
    // explicitly produces the same result as passing it again (determinism),
    // and that the two states can produce different cycle_ids (actually used).
    let mut net = Net::new(64, 2, NKInterpretation::AllN, false, 10_000, 1);

    let mut state_a = State::new(64);
    // Populate with two distinct bit patterns.
    for i in 0..64 {
        state_a.set(i, i % 2 == 0);
    }

    // Same initial state twice must give identical results.
    let r1 = net.perform_run(Some(state_a.clone()), None::<&mut StdRng>, true);
    let r2 = net.perform_run(Some(state_a.clone()), None::<&mut StdRng>, true);
    assert!(!r1.max_steps_reached);
    assert_eq!(
        r1.cycle_id, r2.cycle_id,
        "floyd: same initial -> same cycle_id"
    );
    assert_eq!(
        r1.transient, r2.transient,
        "floyd: same initial -> same transient"
    );
}

#[test]
fn net_initial_state_is_respected_hashset() {
    // Mirror of the floyd test for the hashset path.
    let mut net = Net::new(64, 2, NKInterpretation::AllN, false, 10_000, 1);

    let mut state_a = State::new(64);
    for i in 0..64 {
        state_a.set(i, i % 3 == 0);
    }

    let r1 = net.perform_run(Some(state_a.clone()), None::<&mut StdRng>, false);
    let r2 = net.perform_run(Some(state_a.clone()), None::<&mut StdRng>, false);
    assert!(!r1.max_steps_reached);
    assert_eq!(
        r1.cycle_id, r2.cycle_id,
        "hashset: same initial -> same cycle_id"
    );
    assert_eq!(
        r1.transient, r2.transient,
        "hashset: same initial -> same transient"
    );
}

#[test]
fn net_initial_state_floyd_and_hashset_agree() {
    // Passing the same Some(state) to both variants must yield the same cycle_id
    // and cycle_length, mirroring the rng-driven agreement test.
    let mut net = Net::new(128, 2, NKInterpretation::AllN, false, 10_000, 7);

    let mut state = State::new(128);
    for i in 0..128 {
        state.set(i, (i * 7 + 3) % 5 < 2);
    }

    let floyd = net.perform_run(Some(state.clone()), None::<&mut StdRng>, true);
    let hashset = net.perform_run(Some(state.clone()), None::<&mut StdRng>, false);

    assert!(!floyd.max_steps_reached);
    assert!(!hashset.max_steps_reached);
    assert_eq!(
        floyd.cycle_id, hashset.cycle_id,
        "cycle_id mismatch between floyd and hashset"
    );
    assert_eq!(
        floyd.cycle_length, hashset.cycle_length,
        "cycle_length mismatch between floyd and hashset"
    );
    assert_eq!(
        floyd.transient, hashset.transient,
        "transient mismatch between floyd and hashset"
    );
}
