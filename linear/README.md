<p align="center">
	<img src="linear.svg" title="Linear">
</p>

# Tangram Linear

This crate is documented using rustdoc. View the docs for the most recent version at https://docs.rs/tangram_linear or run `cargo doc -p tangram_linear --open` in the root of this repository.

## Benchmarks

Follow the steps below to run benchmarks on your machine.

1. Create a conda environment using the conda config file.

   `conda env create -f benchmarks/conda.yaml`

2. Run the benchmarks, specifying the datasets and libraries you would like to see benchmark results for.

   `cargo run benchmark --datasets higgs, flights --libraries pytorch, sklearn`

   The available datasets are: allstate, boston, census, flights, heart_disease, higgs, and iris.
   The available libraries are pytorch, sklearn, and tangram.
