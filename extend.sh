#!/bin/bash
# run_experiments.sh
# Runs all Kauffman (1969) random genetic net experiments and saves output to extend_output/

set -e
mkdir -p extend

# Reruns some experiments with K = 3

# ==============================================================================
# Experiment 1 — Fig. 3 & 4
# Cycle length histograms and log median cycle length vs log N
# ==============================================================================

# Fig. 3(a) — N=400, all 16 Boolean functions
cargo run --release -- -k 3 -s 1 -x 1 -n 50 -r 100 | tee  extend_output/fig_3_a.csv

# Fig. 3(b) — N=400, exclude tautology & contradiction
cargo run --release -- -k 5 -s 2 -x 1 -n 50 -r 100 -e | tee  extend_output/fig_3_b.csv

# Fig. 4 — both conditions across all net sizes
cargo run --release -- -k 3 -s  3 -x 1 -n 15   -r 100    | tee  extend_output/fig_4_n15_all.csv
cargo run --release -- -k 3 -s  4 -x 1 -n 15   -r 100 -e | tee  extend_output/fig_4_n15_excl.csv
cargo run --release -- -k 3 -s  5 -x 1 -n 20   -r 100    | tee  extend_output/fig_4_n20_all.csv
cargo run --release -- -k 3 -s  6 -x 1 -n 20   -r 100 -e | tee  extend_output/fig_4_n20_excl.csv
cargo run --release -- -k 3 -s  7 -x 1 -n 25  -r 100     | tee  extend_output/fig_4_n25_all.csv
cargo run --release -- -k 3 -s  8 -x 1 -n 25  -r 100 -e  | tee  extend_output/fig_4_n25_excl.csv
cargo run --release -- -k 3 -s  9 -x 1 -n 50  -r 100     | tee  extend_output/fig_4_n50_all.csv
cargo run --release -- -k 3 -s 10 -x 1 -n 50  -r 100 -e  | tee  extend_output/fig_4_n50_excl.csv
