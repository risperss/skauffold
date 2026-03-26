#!/bin/bash
# run_experiments.sh
# Runs all Kauffman (1969) random genetic net experiments and saves output to output/

set -e
mkdir -p output

# ==============================================================================
# Experiment 1 — Fig. 3 & 4
# Cycle length histograms and log median cycle length vs log N
# ==============================================================================

# Fig. 3(a) — N=400, all 16 Boolean functions
cargo run --release -- -s 1 -x 1 -n 400 -r 100 | tee  output/fig_3_a.csv

# Fig. 3(b) — N=400, exclude tautology & contradiction
cargo run --release -- -s 2 -x 1 -n 400 -r 100 -e | tee  output/fig_3_b.csv

# Fig. 4 — both conditions across all net sizes
cargo run --release -- -s 3 -x 1 -n 15   -r 100    | tee  output/fig_4_n15_all.csv
cargo run --release -- -s 4 -x 1 -n 15   -r 100 -e | tee  output/fig_4_n15_excl.csv
cargo run --release -- -s 5 -x 1 -n 50   -r 100    | tee  output/fig_4_n50_all.csv
cargo run --release -- -s 6 -x 1 -n 50   -r 100 -e | tee  output/fig_4_n50_excl.csv
cargo run --release -- -s 7 -x 1 -n 100  -r 100    | tee  output/fig_4_n100_all.csv
cargo run --release -- -s 8 -x 1 -n 100  -r 100 -e | tee  output/fig_4_n100_excl.csv
cargo run --release -- -s 9 -x 1 -n 191  -r 100    | tee  output/fig_4_n191_all.csv
cargo run --release -- -s 10 -x 1 -n 191  -r 100 -e | tee  output/fig_4_n191_excl.csv
cargo run --release -- -s 11 -x 1 -n 400  -r 100    | tee  output/fig_4_n400_all.csv
cargo run --release -- -s 12 -x 1 -n 400  -r 100 -e | tee  output/fig_4_n400_excl.csv
cargo run --release -- -s 13 -x 1 -n 1024 -r 100    | tee  output/fig_4_n1024_all.csv
cargo run --release -- -s 14 -x 1 -n 1024 -r 100 -e -m 4294967296 | tee  output/fig_4_n1024_excl.csv

# ==============================================================================
# Experiment 2 — Fig. 5
# Scattergram of run-in length vs cycle length
# ==============================================================================

cargo run --release -- -s 15 -x 2 -n 400 -r 100 -e | tee  output/fig_5.csv

# ==============================================================================
# Experiment 3 — Section 5.3
# Activity decay toward cycle
# ==============================================================================

cargo run --release -- -s 16 -x 3 -n 100 -r 50 | tee  output/sec_5_3_activity.csv

# ==============================================================================
# Experiment 4 — Fig. 6 & 7
# Number of cycles per net
# ==============================================================================

# Fig. 6 — N=400, exclude tautology & contradiction
cargo run --release -- -s 17 -x 4 -n 400 -r 100 -e | tee  output/fig_6.csv

# Fig. 7 — across multiple net sizes
cargo run --release -- -s 18 -x 4 -n 15   -r 100 -e | tee  output/fig_7_n15_excl.csv
cargo run --release -- -s 19 -x 4 -n 50   -r 100 -e | tee  output/fig_7_n50_excl.csv
cargo run --release -- -s 20 -x 4 -n 100  -r 100 -e | tee  output/fig_7_n100_excl.csv
cargo run --release -- -s 21 -x 4 -n 191  -r 100 -e | tee  output/fig_7_n191_excl.csv
cargo run --release -- -s 22 -x 4 -n 400  -r 100 -e | tee  output/fig_7_n400_excl.csv
cargo run --release -- -s 23 -x 4 -n 1024 -r 100 -e | tee  output/fig_7_n1024_excl.csv

# ==============================================================================
# Experiment 5 — Fig. 8
# Minimum distance between cycles
# ==============================================================================

cargo run --release -- -s 24 -x 5 -n 100 -r 100 | tee  output/fig_8.csv

# ==============================================================================
# Experiment 6 — Fig. 9 & 10
# Noise perturbations / Markov transition matrix
# ==============================================================================

# Fig. 9 — N=191 and N=400, exclude tautology & contradiction
cargo run --release -- -s 25 -x 6 -n 191 -r 10 -e | tee  output/fig_9_n191.csv
cargo run --release -- -s 26 -x 6 -n 400 -r 10 -e | tee  output/fig_9_n400.csv

# Fig. 10 — verbose output includes full count and probability matrices
cargo run --release -- -s 27 -x 6 -n 191 -r 10 -e -v | tee  output/fig_10_n191_verbose.csv
cargo run --release -- -s 28 -x 6 -n 400 -r 10 -e -v | tee  output/fig_10_n400_verbose.csv

echo "All experiments complete. Results saved to output/"
