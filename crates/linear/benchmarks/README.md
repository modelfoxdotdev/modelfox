# Tangram Linear Benchmarks

To run the benchmarks:

1. Install the nix package manager with flake support: https://nixos.wiki/wiki/Flakes.

2. Clone the tangram repo and `cd` into it: `git clone git@github.com:tangramdotdev/tangram && cd tangram`.

3. Download the datasets:

```
mkdir data
curl -sSL https://datasets.tangram.dev/allstate_train.csv > data/allstate_train.csv
curl -sSL https://datasets.tangram.dev/allstate_test.csv > data/allstate_test.csv
curl -sSL https://datasets.tangram.dev/flights_train.csv > data/flights_train.csv
curl -sSL https://datasets.tangram.dev/flights_test.csv > data/flights_test.csv
curl -sSL https://datasets.tangram.dev/higgs_train.csv > data/higgs_train.csv
curl -sSL https://datasets.tangram.dev/higgs_test.csv > data/higgs_test.csv
```

4. Run `nix develop` to enter the dev shell.

5. Run the benchmarks:

```
cargo run --release -p tangram_linear_benchmarks -- \
  --libraries tangram pytorch sklearn \
    --datasets allstate flights higgs
```
