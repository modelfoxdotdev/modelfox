# Tangram Tree Benchmarks

1.  Install dependencies.

    **Linux**

    - [Rust](https://rustup.rs)
    - (if using nix)

      - [Nix](https://nixos.org/download.html)

        Tangram Nix benchmarks require Nix with flake support, an upcoming feature in Nix. Follow these instructions to enable flakes: https://nixos.wiki/wiki/Flakes.

    - (if using conda)
      - [Conda](https://conda.io)

    **MacOS**

    - [Rust](https://rustup.rs)
    - [Conda](https://conda.io)
    - GNU Time: `brew install gnu-time`.

2.  Clone the Tangram Repo

    `git clone git@github.com:tangramxyz/tangram.git`

3.  Create a `data` directory at the root of the tangram repo.

4.  Download the datasets and place them into the `data` directory you just created.

- Allstate:

  - http://datasets.tangram.xyz/allstate_test.csv
  - http://datasets.tangram.xyz/allstate_train.csv

- Flights:

  - http://datasets.tangram.xyz/flights_test.csv
  - http://datasets.tangram.xyz/flights_train.csv

- Higgs:
  - http://datasets.tangram.xyz/higgs_test.csv
  - http://datasets.tangram.xyz/higgs_train.csv

5.  Set up the environment:

    **MacOS or Linux using Conda**:

    1.  Create a new Conda environment:

        `conda env create --file conda.yml`

        This will create a new conda environment called `tangram_tree_benchmarks`

    2.  Activate the conda environment:

        `conda activate tangram_tree_benchmarks`

6.  Run the benchmarks!

    Each of the commands should be run from the root of the tangram repo.

    **MacOS or Linux using Conda**:

    ```
    cargo run --release --bin tangram_tree_benchmarks -- \
    --libraries tangram lightgbm xgboost \
    --datasets higgs flights allstate
    ```

    **Linux using Nix**:

    ```
    nix develop ./tree/benchmarks -c cargo run --release --bin tangram_tree_benchmarks -- \
    --libraries tangram xgboost sklearn lightgbm \
    --datasets higgs flights allstate
    ```
