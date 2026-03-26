import itertools
import subprocess


def main():
    subprocess.run(["cargo", "build", "--release"])

    output_dir = "stability_output"
    runs = 500

    n_vals = [15, 50, 100, 191, 400, 1024, 2048]
    k_vals = [2, 3, 4]
    exclude_taut_and_cond_vals = ["", "-e"]

    for i, (n, k, e) in enumerate(
        itertools.product(n_vals, k_vals, exclude_taut_and_cond_vals)
    ):
        subprocess.run(
            " ".join(
                [
                    "./target/release/skauffold -x 1 -m 1024",
                    f"-s {i} -n {n} -r {runs} {e}",
                    f"| tee {output_dir}/fig_4_n{n}_k{k}_{'excl' if e else 'all'}.csv",
                ]
            ),
            shell=True,
            check=True,
        )


if __name__ == "__main__":
    main()
