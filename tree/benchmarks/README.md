# Tangram Tree Benchmarks

1. Install dependencies.

- [Rust](https://rustup.rs)
- [Conda](https://conda.io)
- GNU Time.
	- linux: installed by default.
	- macos: `brew install gnu-time`.
	- windows: `scoop install time`.

2. Create and activate the conda environment:

```
conda env create -f conda.yaml
conda activate tangram_tree_benchmarks
```

3. Run the benchmarks:

```
cargo run --release --bin tangram_tree_benchmarks -- \
--libraries tangram lightgbm \
--datasets higgs flights allstate
```
