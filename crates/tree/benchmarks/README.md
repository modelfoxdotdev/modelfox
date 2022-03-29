# ModelFox Tree Benchmarks

To run the benchmarks:

1. Install the nix package manager with flake support: https://nixos.wiki/wiki/Flakes.

2. Clone the modelfox repo and `cd` into it: `git clone git@github.com:modelfoxdotdev/modelfox && cd modelfox`.

3. Download the datasets:

```
mkdir data
curl -sSL https://datasets.modelfox.dev/allstate_train.csv > data/allstate_train.csv
curl -sSL https://datasets.modelfox.dev/allstate_test.csv > data/allstate_test.csv
curl -sSL https://datasets.modelfox.dev/flights_train.csv > data/flights_train.csv
curl -sSL https://datasets.modelfox.dev/flights_test.csv > data/flights_test.csv
curl -sSL https://datasets.modelfox.dev/higgs_train.csv > data/higgs_train.csv
curl -sSL https://datasets.modelfox.dev/higgs_test.csv > data/higgs_test.csv
```

4. Run `nix develop` to enter the dev shell.

5. Run the benchmarks:

```
cargo run --release -p modelfox_tree_benchmarks -- \
  --libraries modelfox lightgbm xgboost \
  --datasets allstate flights higgs
```
