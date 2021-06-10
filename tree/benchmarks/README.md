# Tangram Tree Benchmarks

To run the benchmarks:

1. Install the nix package manager with flake support: https://nixos.wiki/wiki/Flakes.

2. Clone the tangram repo and `cd` into it: `git clone git@github.com:tangramxyz/tangram && cd tangram`.

3. Download the datasets:

```
mkdir data
curl -sSL http://datasets.tangram.xyz/allstate_train.csv > data/allstate_train.csv
curl -sSL http://datasets.tangram.xyz/allstate_test.csv > data/allstate_test.csv
curl -sSL http://datasets.tangram.xyz/flights_train.csv > data/flights_train.csv
curl -sSL http://datasets.tangram.xyz/flights_test.csv > data/flights_test.csv
curl -sSL http://datasets.tangram.xyz/higgs_train.csv > data/higgs_train.csv
curl -sSL http://datasets.tangram.xyz/higgs_test.csv > data/higgs_test.csv
```

4. Run `nix develop` to enter the dev shell.

5. Run the benchmarks:

```
cargo run --release --bin tangram_tree_benchmarks -- \
  --libraries tangram lightgbm xgboost \
  --datasets allstate flights higgs
```
