mod net;
use std::collections::HashSet;

use clap::{Parser, ValueEnum};
use net::Net;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::net::State;

#[derive(Debug, Clone, ValueEnum)]
enum Experiment {
    Experiment5Dot1,
    Experiment5Dot2,
    Experiment5Dot3,
    Experiment5Dot4,
    Experiment5Dot5,
    Experiment5Dot6,
}

/// Boolean network attractor simulation
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Which experiment to run (runs basic simulation if not specified)
    #[arg(short = 'x', long, value_enum)]
    experiment: Option<Experiment>,

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
    #[arg(short = 'm', long, default_value_t = 10_000)]
    max_steps: usize,

    /// Print per-run details
    #[arg(short = 'v', long, default_value_t = false)]
    verbose: bool,
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
        "  n={}, k={}, exclude_taut_and_cont={}, runs={}, seed={}, max_steps={}",
        args.num_nodes,
        args.num_inputs,
        args.exclude_taut_and_cont,
        args.runs,
        args.seed,
        args.max_steps,
    );
    eprintln!();
}

fn main() {
    let args = Args::parse();
    printargs(&args);

    match args.experiment {
        Some(Experiment::Experiment5Dot1) => run_experiment_5_dot_1(&args),
        Some(Experiment::Experiment5Dot2) => run_experiment_5_dot_2(&args),
        Some(Experiment::Experiment5Dot3) => run_experiment_5_dot_3(&args),
        Some(Experiment::Experiment5Dot4) => run_experiment_5_dot_4(&args),
        Some(Experiment::Experiment5Dot5) => run_experiment_5_dot_5(&args),
        Some(Experiment::Experiment5Dot6) => run_experiment_5_dot_6(&args),
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
            args.exclude_taut_and_cont,
            args.max_steps,
            net_seed,
        );
        let mut run_rng = StdRng::seed_from_u64(run_seed);
        let result = net.perform_run(&mut run_rng);

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
                    "Run {:>4}: transient={:>6}  cycle_length={:>6}",
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
 * FIG. 3. (a) A histogram of the lengths of state cycles in nets of 400 binary elements which
 * used all 16 Boolean functions of two variables equiprobably. The distribution is skewed
 * toward short cycles. (b) A histogram of the lengths of state cycles in nets of 400 binary
 * elements which used neither tautology nor contradiction, but used the remaining 14
 * Boolean functions of 2 variables equiprobably. The distribution is skewed toward short
 * cycles.
 *
 * FIG. 4. Log median cycle length as a function of log N, in nets using all 16 Boolean
 * functions of two inputs (all Boolean functions used), and in nets disallowing these two func-
 * tions (tautology and contradiction not used). The asymptotic slopes are about 0.3 and 0.6.
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
            args.exclude_taut_and_cont,
            args.max_steps,
            net_seed,
        );
        let result = net.perform_run(&mut run_rng);
        println!("{},{}", result.cycle_length, result.max_steps_reached);
    }
}

/**
 * FIG. 5. A scattergram of run-in length and cycle length in nets of 400 binary elements
 * using neither tautology nor contradiction. Run-in length appears uncorrelated with cycle
 * length. A log/log plot was used merely to accommodate the data.
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
            args.exclude_taut_and_cont,
            args.max_steps,
            net_seed,
        );
        let result = net.perform_run(&mut run_rng);
        println!(
            "{},{},{}",
            result.cycle_length, result.transient, result.max_steps_reached
        );
    }
}

/**
* When the system is released from an arbitary initial state, the number of
* elements which change value (the activity) per state transition decreases
* rapidly. In nets of 100 elements, using all 16 Boolean functions, the number
* of elements which change value at the first state transition is about 0.4N
* This decreases, along a curve nearly fitted by a negative exponential with a
* half decay of 3-4 state transitions, to a minimum activity of 0 to 0.25N per
* state transition along the cycle. For larger nets, the half decay should require
* more transitions. Thus, as the system approaches a cycle, states become
* progressively more similar. One would expect that all states which differ from
* cycle states in the value of only one element would themselves be located a
* very few state transitions from that cycle.
* The number of genes which change value during a cycle varies between 0
* and 35 in nets of 100 elements using all 16 Boolean functions. The consequence
* is that most genes are constant throughout the cycle, and the cycle states are
* highly similar.
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
            args.exclude_taut_and_cont,
            args.max_steps,
            net_seed,
        );
        let result = net.perform_run(&mut run_rng);
        if result.max_steps_reached {
            print!("MAX_STEPS_REACHED,")
        }
        // println!(
        //     "{}",
        //     result
        //         .pairwise_hamming_distances()
        //         .iter()
        //         .map(|x| x.to_string())
        //         .collect::<Vec<_>>()
        //         .join(",")
        // );
    }
}

/**
 * FIG. 6. A histogram of the number of cycles per net in nets of 400 elements using neither
 * tautology nor contradiction, but the remaining Boolean functions of two inputs equiprobably.
 * The median is 10 cycles per net. The distribution is skewed toward few cycles.
 *
 * FIG. 7. The median number of cycles per net as N increases appears linear in a log/log
 * plot. The slope is about 0.3. The expected number of cycles is slightly less than square root N.
 */
fn run_experiment_5_dot_4(args: &Args) {
    let mut rng = StdRng::seed_from_u64(args.seed);

    // for a given net size
    println!("Nodes: {:>4}", args.num_nodes);
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
            args.exclude_taut_and_cont,
            args.max_steps,
            net_seed,
        );
        // 50 times per net
        for _ in 0..50 {
            let run_seed: u64 = rng.r#gen();
            let mut run_rng = StdRng::seed_from_u64(run_seed);
            let result = net.perform_run(&mut run_rng);
            if let Some(cycle_id) = result.cycle_id {
                cycle_ids.insert(cycle_id);
            } else {
                max_steps_reached += 1;
            }
        }
        let num_unique_cycles = cycle_ids.len();
        println!(
            "Net: {:>2} had {:>4} unique cycles, {:>4} max steps reached",
            run, num_unique_cycles, max_steps_reached
        );
        num_cycles.push(num_unique_cycles);
        num_cycles_incl_msr.push(num_unique_cycles + max_steps_reached);
    }
    // print the median number of cycles found in a net of that size
    println!(
        "{}",
        num_cycles
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );
    println!(
        "{}",
        num_cycles_incl_msr
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );
    println!(
        "Median number of cycles for an n={:<4} net: {:>4}, (incl msr): {:>4}",
        args.num_nodes,
        median(&mut num_cycles),
        median(&mut num_cycles_incl_msr),
    )
}

fn run_experiment_5_dot_5(_args: &Args) {}

/**
 * FIG. 8. A scattergram of the minimum distance between cycles and cycle length in nets of
 * 100 elements using all 16 Boolean functions of two variables. Minimum distance between
 * cycles appears uncorrelated with cycle length. The median minimum distance is 0.05N.
 *
 * FIG. 9. (a) The total number of cycles reached from each cycle after it was perturbed in
 * all possible ways by one unit of noise correlated with the number of cycles in the net being
 * perturbed. The data is from nets using neither tautology nor contradiction, with N = 191,
 * and 400. (b) The number of cycles reached from each cycle with a probability greater than
 * 0.01 in the same nets as those of (a). In nets using all 16 Boolean functions, the total number
 * of cycles reached from each cycle is about the same as the data in (b).
 */
fn run_experiment_5_dot_6(_args: &Args) {}
