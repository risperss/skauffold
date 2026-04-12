import itertools
import subprocess


def main():
    subprocess.run(["cargo", "build", "--release"])

    output_dir = "stability_output"
    max_val = 1 << 10
    runs = 100

    n_vals = [15, 20, 25, 50, 100, 191, 400, 1024, 8192]
    e_vals = ["", "-e"]
    k_vals = [2, 3, 4]
    i_vals = ["all-n", "all-n-except-self", "self-plus-other-n"]

    for i, (n, k, e, i_type) in enumerate(
        itertools.product(n_vals, k_vals, e_vals, i_vals)
    ):
        subprocess.run(
            " ".join(
                [
                    "./target/release/skauffold -x 1",
                    f"-m {max_val} -s {i} -k {k} -n {n} -r {runs} -i {i_type} {e}",
                    f"| tee {output_dir}/fig_4_n{n}_k{k}_{'excl' if e else 'all'}_{i_type}.csv",
                ]
            ),
            shell=True,
            check=True,
        )


if __name__ == "__main__":
    main()
