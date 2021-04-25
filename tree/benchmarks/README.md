# Tangram Tree Benchmarks

To run the benchmarks you will need:
- Rust: `https://www.rust-lang.org/tools/install`
- Python and Conda
- Linux OS: memory usage is measured by reading `VmHWM` from `/proc/self/status`.

1. Create the conda environment which contains the python gradient boosted decision tree implementations from LightGBM, XGBoost, CatBoost, H2O, and scikit-learn:

	```conda env create -f conda.yaml```
2. Activate the conda environment:

	```conda activate tangram_tree_benchmarks```
3. Run the benchmarks:

	```cargo run --release --bin tangram_tree_benchmarks -- --libraries tangram lightgbm --datasets higgs flights allstate```
