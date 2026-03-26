mod net;
mod util;

use core::fmt;
use std::collections::{HashMap, HashSet};

use clap::{Parser, ValueEnum};
use net::Net;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::util::State;

#[derive(ValueEnum, Clone, Copy, Debug, Default)]
#[clap(rename_all = "kebab-case")]
enum NKInterpretation {
    #[default]
    AllN,
    AllNExceptSelf,
    SelfPlusOtherN,
}

impl fmt::Display for NKInterpretation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NKInterpretation::AllN => write!(f, "all-n"),
            NKInterpretation::AllNExceptSelf => write!(f, "all-n-except-self"),
            NKInterpretation::SelfPlusOtherN => write!(f, "self-plus-other-n"),
        }
    }
}

/// Random boolean network simulation
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Which experiment to run (runs basic simulation if not specified)
    #[arg(short = 'x', long, value_enum)]
    experiment: Option<u8>,

    /// Number of nodes in the network
    #[arg(short = 'n', long, default_value_t = 400)]
    num_nodes: usize,

    /// Number of inputs per node
    #[arg(short = 'k', long, default_value_t = 2)]
    num_inputs: u32,

    /// Exclude tautology and contradiction as possible functions
    #[arg(short = 'e', long, default_value_t = false)]
    exclude_taut_and_cont: bool,

    /// Number of runs per experiment
    #[arg(short = 'r', long, default_value_t = 50)]
    runs: usize,

    /// Random seed
    #[arg(short = 's', long, default_value_t = 537)]
    seed: u64,

    /// Maximum number of steps to explore per run
    #[arg(short = 'm', long, default_value_t = 1 << 16)]
    max_steps: usize,

    /// Print per-run details
    #[arg(short = 'v', long, default_value_t = false)]
    verbose: bool,

    /// Number of random starting points in various experiments
    #[arg(short = 'p', long, default_value_t = 50)]
    start_points_per_net: usize,

    /// Interpretation of NK networks
    #[arg(short = 'i', long, default_value_t = NKInterpretation::AllN)]
    nk_interpretation: NKInterpretation,
}

fn mean(data: &[f64]) -> f64 {
    data.iter().sum::<f64>() / data.len() as f64
}

fn median<T: PartialOrd + Copy>(data: &mut [T]) -> T {
    data.sort_by(|a, b| a.partial_cmp(b).unwrap());
    data[data.len() / 2]
}

fn stdev(data: &[f64]) -> f64 {
    let m = mean(data);
    (data.iter().map(|x| (x - m).powi(2)).sum::<f64>() / data.len() as f64).sqrt()
}

fn printargs(args: &Args) {
    eprintln!("Boolean Network Simulation");
    eprintln!(
        "  n={}, k={}, exclude_taut_and_cont={}, runs={}, seed={}, max_steps={}, start_points={}, nk-interpretation={}",
        args.num_nodes,
        args.num_inputs,
        args.exclude_taut_and_cont,
        args.runs,
        args.seed,
        args.max_steps,
        args.start_points_per_net,
        args.nk_interpretation,
    );
    eprintln!();
}

fn main() {
    let args = Args::parse();
    printargs(&args);

    match args.experiment {
        Some(1) => run_experiment_5_dot_1(&args),
        Some(2) => run_experiment_5_dot_2(&args),
        Some(3) => run_experiment_5_dot_3(&args),
        Some(4) => run_experiment_5_dot_4(&args),
        Some(5) => run_experiment_5_dot_5(&args),
        Some(6) => run_experiment_5_dot_6(&args),
        _ => default(&args),
    }
}

fn default(args: &Args) {
    // Top-level RNG used to derive per-run seeds, ensuring full reproducibility.
    let mut rng = StdRng::seed_from_u64(args.seed);

    let mut transients: Vec<f64> = Vec::new();
    let mut cycle_lengths: Vec<f64> = Vec::new();
    let mut max_steps_count = 0usize;

    for run in 0..args.runs {
        let net_seed: u64 = rng.r#gen();
        let run_seed: u64 = rng.r#gen();
        let mut net = Net::new(
            args.num_nodes,
            args.num_inputs,
            args.nk_interpretation,
            args.exclude_taut_and_cont,
            args.max_steps,
            net_seed,
        );
        let mut run_rng = StdRng::seed_from_u64(run_seed);
        let result = net.perform_run(None, Some(&mut run_rng), true);

        if result.max_steps_reached {
            max_steps_count += 1;
            if args.verbose {
                println!(
                    "Run {:>4}: max_steps reached (>{} steps)",
                    run + 1,
                    args.max_steps
                );
            }
        } else {
            if args.verbose {
                println!(
                    "Run {:>4}: transient={:>8}  cycle_length={:>8}",
                    run + 1,
                    result.transient,
                    result.cycle_length,
                );
            }
            transients.push(result.transient as f64);
            cycle_lengths.push(result.cycle_length as f64);
        }
    }

    println!("Results ({} runs):", args.runs);
    println!("  max_steps_reached : {}", max_steps_count);
    if !transients.is_empty() {
        println!(
            "  cycle_length      : median={:.2}  mean={:.2}  stdev={:.2}",
            median(&mut cycle_lengths),
            mean(&cycle_lengths),
            stdev(&cycle_lengths),
        );
        println!(
            "  transient         : median={:.2}  mean={:.2}  stdev={:.2}",
            median(&mut transients),
            mean(&transients),
            stdev(&transients),
        );
    } else {
        println!("  (all runs hit max_steps — no cycle statistics available)");
    }
}

/**
FIG. 3.
(a) A histogram of the lengths of state cycles in nets of 400 binary elements which
used all 16 Boolean functions of two variables equiprobably. The distribution is skewed
toward short cycles.
(b) A histogram of the lengths of state cycles in nets of 400 binary
elements which used neither tautology nor contradiction, but used the remaining 14
Boolean functions of 2 variables equiprobably. The distribution is skewed toward short
cycles.

FIG. 4. Log median cycle length as a function of log N, in nets using all 16 Boolean
functions of two inputs (all Boolean functions used), and in nets disallowing these two func-
tions (tautology and contradiction not used). The asymptotic slopes are about 0.3 and 0.6.
*/
fn run_experiment_5_dot_1(args: &Args) {
    let mut rng = StdRng::seed_from_u64(args.seed);

    println!("cycle_length,max_steps_reached");

    for _run in 0..args.runs {
        let net_seed: u64 = rng.r#gen();
        let run_seed: u64 = rng.r#gen();
        let mut run_rng = StdRng::seed_from_u64(run_seed);
        let mut net = Net::new(
            args.num_nodes,
            args.num_inputs,
            args.nk_interpretation,
            args.exclude_taut_and_cont,
            args.max_steps,
            net_seed,
        );
        let result = net.perform_run(None, Some(&mut run_rng), true);
        println!(
            "{},{}",
            if result.max_steps_reached {
                // allows for slightly easier data processing
                args.max_steps
            } else {
                result.cycle_length
            },
            result.max_steps_reached
        );
    }
}

/**
FIG. 5. A scattergram of run-in length and cycle length in nets of 400 binary elements
using neither tautology nor contradiction. Run-in length appears uncorrelated with cycle
length. A log/log plot was used merely to accommodate the data.
*/
fn run_experiment_5_dot_2(args: &Args) {
    let mut rng = StdRng::seed_from_u64(args.seed);

    println!("cycle_length,transient,max_steps_reached");

    for _run in 0..args.runs {
        let net_seed: u64 = rng.r#gen();
        let run_seed: u64 = rng.r#gen();
        let mut run_rng = StdRng::seed_from_u64(run_seed);
        let mut net = Net::new(
            args.num_nodes,
            args.num_inputs,
            args.nk_interpretation,
            args.exclude_taut_and_cont,
            args.max_steps,
            net_seed,
        );
        let result = net.perform_run(None, Some(&mut run_rng), true);
        println!(
            "{},{},{}",
            result.cycle_length, result.transient, result.max_steps_reached
        );
    }
}

/**
When the system is released from an arbitary initial state, the number of
elements which change value (the activity) per state transition decreases
rapidly. In nets of 100 elements, using all 16 Boolean functions, the number
of elements which change value at the first state transition is about 0.4N
This decreases, along a curve nearly fitted by a negative exponential with a
half decay of 3-4 state transitions, to a minimum activity of 0 to 0.25N per
state transition along the cycle. For larger nets, the half decay should require
more transitions. Thus, as the system approaches a cycle, states become
progressively more similar. One would expect that all states which differ from
cycle states in the value of only one element would themselves be located a
very few state transitions from that cycle.
The number of genes which change value during a cycle varies between 0
and 35 in nets of 100 elements using all 16 Boolean functions. The consequence
is that most genes are constant throughout the cycle, and the cycle states are
highly similar.
*/
fn run_experiment_5_dot_3(args: &Args) {
    let mut rng = StdRng::seed_from_u64(args.seed);

    for _run in 0..args.runs {
        let net_seed: u64 = rng.r#gen();
        let run_seed: u64 = rng.r#gen();
        let mut run_rng = StdRng::seed_from_u64(run_seed);
        let mut net = Net::new(
            args.num_nodes,
            args.num_inputs,
            args.nk_interpretation,
            args.exclude_taut_and_cont,
            args.max_steps,
            net_seed,
        );
        let result = net.perform_run(None, Some(&mut run_rng), false);
        if let Some(states) = result.states {
            println!("{}, {}", result.transient, result.cycle_length);
            println!(
                "{}",
                pairwise_hamming_distances(states)
                    .iter()
                    .map(|x| (*x as f64 / args.num_nodes as f64).to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            );
        }
    }
}

pub fn pairwise_hamming_distances(states: Vec<State>) -> Vec<usize> {
    states
        .windows(2)
        .map(|pair| pair[0].hamming_distance(&pair[1]))
        .collect()
}

/**
FIG. 6. A histogram of the number of cycles per net in nets of 400 elements using neither
tautology nor contradiction, but the remaining Boolean functions of two inputs equiprobably.
The median is 10 cycles per net. The distribution is skewed toward few cycles.
*
FIG. 7. The median number of cycles per net as N increases appears linear in a log/log
plot. The slope is about 0.3. The expected number of cycles is slightly less than square root N.
*/
fn run_experiment_5_dot_4(args: &Args) {
    let mut rng = StdRng::seed_from_u64(args.seed);

    println!("unique_cycles,msr");
    let mut num_cycles: Vec<usize> = Vec::with_capacity(args.runs);
    let mut num_cycles_incl_msr: Vec<usize> = Vec::with_capacity(args.runs);
    // across some number of runs per net size
    for run in 0..args.runs {
        let net_seed: u64 = rng.r#gen();

        // Paper's experiment uses 50 successive runs per net
        let mut cycle_ids: HashSet<State> = HashSet::with_capacity(50);
        let mut max_steps_reached = 0;
        let mut net = Net::new(
            args.num_nodes,
            args.num_inputs,
            args.nk_interpretation,
            args.exclude_taut_and_cont,
            args.max_steps,
            net_seed,
        );
        // 50 times per net
        for _ in 0..args.start_points_per_net {
            let run_seed: u64 = rng.r#gen();
            let mut run_rng = StdRng::seed_from_u64(run_seed);
            let result = net.perform_run(None, Some(&mut run_rng), true);
            if let Some(cycle_id) = result.cycle_id {
                cycle_ids.insert(cycle_id);
            } else {
                max_steps_reached += 1;
            }
        }
        let num_unique_cycles = cycle_ids.len();
        if args.verbose {
            eprintln!(
                "Net: {:>2} had {:>4} unique cycles, {:>4} max steps reached",
                run + 1,
                num_unique_cycles,
                max_steps_reached
            );
        }
        println!("{},{}", num_unique_cycles, max_steps_reached);
        num_cycles.push(num_unique_cycles);
        num_cycles_incl_msr.push(num_unique_cycles + max_steps_reached);
    }
    // print the median number of cycles found in a net of that size
    eprintln!("median={}", median(&mut num_cycles),)
}

/**
The minimum possible difference between states on two distinct cycles is 1
-a difference in the value of a single element. This distance occurs frequently
but the minimum distance may be as large as 0.3N. Figure 8 is a scattergram
of minimum distances between cycles correlated with the length of the cycles
in many nets of 100 elements using all 16 Boolean functions. The median
minimum distance between cycles is 5. The average distance between cycles
is about 10. When a net embodies many cycles, these frequently form sets
within which each cycle is a minimum distance of one from one or two
members of the set. Between sets, the distance is larger and may be as great
as 0*3N.

FIG. 8. A scattergram of the minimum distance between cycles and cycle length in nets of
100 elements using all 16 Boolean functions of two variables. Minimum distance between
cycles appears uncorrelated with cycle length. The median minimum distance is 0.05N.
*/
fn run_experiment_5_dot_5(args: &Args) {
    let mut rng = StdRng::seed_from_u64(args.seed);

    println!("cycle_length,min_distance,net");

    for net_idx in 0..args.runs {
        let net_seed: u64 = rng.r#gen();
        let mut net = Net::new(
            args.num_nodes,
            args.num_inputs,
            args.nk_interpretation,
            args.exclude_taut_and_cont,
            args.max_steps,
            net_seed,
        );

        let mut cycles: HashMap<State, Vec<State>> = HashMap::new();

        for _ in 0..50 {
            let run_seed: u64 = rng.r#gen();
            let mut run_rng = StdRng::seed_from_u64(run_seed);

            let result = net.perform_run(None, Some(&mut run_rng), false);

            if let (Some(cycle_id), Some(states)) = (result.cycle_id, result.states) {
                cycles
                    .entry(cycle_id)
                    .or_insert_with(|| states[result.transient..].to_vec());
            }
            // Runs that hit max_steps are silently skipped: we have no cycle
            // states to work with.
        }

        // Need at least two distinct cycles to compute an inter-cycle distance.
        if cycles.len() < 2 {
            continue;
        }

        // Collect into an indexed vec so we can do pairwise iteration cleanly.
        let cycle_list: Vec<(&State, &Vec<State>)> = cycles.iter().collect();
        let num_cycles = cycle_list.len();

        for i in 0..num_cycles {
            let (_, states_i) = cycle_list[i];
            let cycle_len_i = states_i.len();

            // Minimum Hamming distance from cycle i to any other cycle j.
            let mut min_dist = usize::MAX;
            'outer: for j in 0..num_cycles {
                if i == j {
                    continue;
                }
                let (_, states_j) = cycle_list[j];

                for si in states_i {
                    for sj in states_j {
                        let d = si.hamming_distance(sj);
                        if d < min_dist {
                            min_dist = d;
                        }

                        if min_dist == 0 {
                            break 'outer;
                        }
                    }
                }
            }

            // Normalise by N so the result is comparable with the paper's
            // "0.05N" median figure regardless of the --num-nodes setting.
            let normalised = min_dist as f64 / args.num_nodes as f64;
            println!("{},{:.6},{}", cycle_len_i, normalised, net_idx + 1);
        }
    }
}

/**
The effect of state noise on the behavior of K = 2 random nets has been
studied by perturbing the system as it traverses a cycle by arbitrarily reversing
the value of a single gene for a single time moment. The perturbed net may
either return to the behavior cycle from which it was dislodged, or run in to
a different cycle. The program first built a net, then explored it from 50
randomly chosen initial states, and stored the different state cycles discovered.
Then all states which differed by the value of one gene from each state of the
first cycle discovered were tried, and the cycle to which each of these states
ran was stored. From this, a row listing the number of times perturbation
by one unit of noise shifted the system from the first behavior cycle to each of
the cycles was compiled. The procedure was repeated for all remaining
cycles, generating a square matrix listing of the transitions between cycles
induced by all possible single units of noise. Division of the number in each
cell of the matrix by the row total results in a matrix of transition probabilities
under the drive of random (1 unit) noise, which is a Markov chain (see Fig. 10).

FIG. 9. (a) The total number of cycles reached from each cycle after it was perturbed in
all possible ways by one unit of noise correlated with the number of cycles in the net being
perturbed. The data is from nets using neither tautology nor contradiction, with N = 191,
and 400. (b) The number of cycles reached from each cycle with a probability greater than
0.01 in the same nets as those of (a). In nets using all 16 Boolean functions, the total number
of cycles reached from each cycle is about the same as the data in (b).
*/
fn run_experiment_5_dot_6(args: &Args) {
    let mut rng = StdRng::seed_from_u64(args.seed);

    // eh, if they pass verbose just output the matrix to stdout. makes life so much easier
    if !args.verbose {
        println!(
            "net,num_cycles,cycle_idx,cycle_length,total_reachable,reachable_above_0.01,unknown_count,timeout_count"
        );
    }

    for net_idx in 0..args.runs {
        let net_seed: u64 = rng.r#gen();
        let mut net = Net::new(
            args.num_nodes,
            args.num_inputs,
            args.nk_interpretation,
            args.exclude_taut_and_cont,
            args.max_steps,
            net_seed,
        );

        // ── Phase 1: discover cycles ─────────────────────────────────────────
        let mut cycles: HashMap<State, Vec<State>> = HashMap::new();

        for _ in 0..args.start_points_per_net {
            let run_seed: u64 = rng.r#gen();
            let mut run_rng = StdRng::seed_from_u64(run_seed);
            // Hashset variant needed: only one that returns the state trajectory.
            let result = net.perform_run(None, Some(&mut run_rng), false);

            if let (Some(cycle_id), Some(states)) = (result.cycle_id, result.states) {
                cycles
                    .entry(cycle_id)
                    .or_insert_with(|| states[result.transient..].to_vec());
            }
        }

        // A single-cycle net produces a trivially diagonal matrix — no
        // inter-cycle transitions are possible. Skip it.
        if cycles.len() < 2 {
            continue;
        }

        // Assign stable integer indices by sorting on cycle_id (the lex-min
        // on-cycle state), guaranteeing identical ordering across runs with
        // the same seed.
        let mut cycle_list: Vec<(State, Vec<State>)> = cycles.into_iter().collect();
        cycle_list.sort_by(|(a, _), (b, _)| a.cmp(b));
        let num_cycles = cycle_list.len();

        // Reverse lookup: cycle_id -> column index in the matrix.
        let cycle_index: HashMap<&State, usize> = cycle_list
            .iter()
            .enumerate()
            .map(|(i, (id, _))| (id, i))
            .collect();

        // ── Phase 2: perturbation sweep ──────────────────────────────────────
        // matrix[i][j]: how many single-bit perturbations of any state on
        //               cycle i lead to cycle j.
        // unknown_counts[i]: perturbations that landed on a cycle not found
        //                    in phase 1 (undercounting of cycles is possible
        //                    with only 50 starts).
        // timeout_counts[i]: perturbations that hit max_steps.
        let mut matrix = vec![vec![0usize; num_cycles]; num_cycles];
        let mut unknown_counts = vec![0usize; num_cycles];
        let mut timeout_counts = vec![0usize; num_cycles];

        for (i, (_, cycle_states)) in cycle_list.iter().enumerate() {
            for state in cycle_states {
                for bit in 0..args.num_nodes {
                    // Clone once and flip the target bit.
                    let mut perturbed = state.clone();
                    let current = perturbed.get(bit as u16);
                    perturbed.set(bit, !current);

                    // Floyd is O(1) memory and we only need the cycle_id here.
                    let result = net.perform_run(Some(perturbed), None::<&mut StdRng>, true);

                    if result.max_steps_reached {
                        timeout_counts[i] += 1;
                    } else if let Some(dest_id) = result.cycle_id {
                        if let Some(&j) = cycle_index.get(&dest_id) {
                            matrix[i][j] += 1;
                        } else {
                            // A cycle that exists in the net but wasn't
                            // discovered during the 50-start phase 1 sweep.
                            unknown_counts[i] += 1;
                        }
                    }
                }
            }
        }

        // ── Phase 3: normalise and emit ──────────────────────────────────────
        if args.verbose {
            eprintln!("Net {:>3}: {} cycles", net_idx + 1, num_cycles);
            eprintln!("  Raw count matrix (rows = source cycle, cols = dest cycle):");
            for row in &matrix {
                let cells: Vec<String> = row.iter().map(|x| format!("{}", x)).collect();
                println!("{}", cells.join(","));
            }
        }

        for (i, (_, cycle_states)) in cycle_list.iter().enumerate() {
            let row_total: usize = matrix[i].iter().sum();
            let cycle_length = cycle_states.len();

            // Fig 9a: total distinct destination cycles reached (prob > 0).
            let total_reachable = matrix[i].iter().filter(|&&x| x > 0).count();

            // Fig 9b: destination cycles reached with probability > 0.01.
            let reachable_above_001 = if row_total > 0 {
                matrix[i]
                    .iter()
                    .filter(|&&x| x as f64 / row_total as f64 > 0.01)
                    .count()
            } else {
                0
            };

            if !args.verbose {
                println!(
                    "{},{},{},{},{},{},{},{}",
                    net_idx + 1,
                    num_cycles,
                    i,
                    cycle_length,
                    total_reachable,
                    reachable_above_001,
                    unknown_counts[i],
                    timeout_counts[i],
                );
            }
        }
    }
}
