<p align="center">
	<img width="200px" src="modelfox.png" title="ModelFox">
</p>

<h1 align="center">
ModelFox makes it easy to train, deploy, and monitor machine learning models.
</h1>

<p align="center">
Train a model from a CSV file on the command line. Make predictions from Elixir, Go, JavaScript, PHP, Python, Ruby, or Rust. Learn about your models and monitor them in production from your browser.
</p>

<p align="center">
	<a href="https://twitter.com/intent/follow?screen_name=modelfoxdotdev">
		<img src="https://img.shields.io/twitter/follow/modelfoxdotdev?label=Follow%20modelfoxdotdev&style=social&color=blue" alt="Follow @modelfoxdotdev on Twitter" />
	</a>
</p>

# ModelFox

[Discord](https://discord.gg/jT9ZGp3TK2)

ModelFox makes it easy to train, deploy, and monitor machine learning models.

- Run `modelfox train` to train a model from a CSV file on the command line.
- Make predictions with libraries for [Elixir](https://hex.pm/packages/modelfox), [Go](https://pkg.go.dev/github.com/modelfoxdotdev/modelfox-go), [JavaScript](https://www.npmjs.com/package/@modelfoxdotdev/modelfox), [PHP](https://packagist.org/packages/modelfox/modelfox), [Python](https://pypi.org/project/modelfox), [Ruby](https://rubygems.org/gems/modelfox), and [Rust](https://lib.rs/crates/modelfox).
- Run `modelfox app` to learn more about your models and monitor them in production.

### Install

You can install the modelfox CLI by either downloading the binary from the [latest github release](https://github.com/modelfoxdotdev/modelfox/releases/tag/v0.8.0), or by building from source. 

### Train

Train a machine learning model by running `modelfox train` with the path to a CSV file and the name of the column you want to predict.

```
$ modelfox train --file heart_disease.csv --target diagnosis --output heart_disease.modelfox
✅ Loading data.
✅ Computing features.
🚂 Training model 1 of 8.
[==========================================>                         ]
```

The CLI automatically transforms your data into features, trains a number of linear and gradient boosted decision tree models to predict the target column, and writes the best model to a `.modelfox` file. If you want more control, you can provide a config file.

### Predict

Make predictions with libraries for [Elixir](https://hex.pm/packages/modelfox), [Go](https://pkg.go.dev/github.com/modelfoxdotdev/modelfox-go), [JavaScript](https://www.npmjs.com/package/@modelfoxdotdev/modelfox), [PHP](https://packagist.org/packages/modelfox/modelfox), [Python](https://pypi.org/project/modelfox), [Ruby](https://rubygems.org/gems/modelfox), and [Rust](https://lib.rs/modelfox).

```javascript
let modelfox = require("@modelfoxdotdev/modelfox")

let model = new modelfox.Model("./heart_disease.modelfox")

let input = {
	age: 63,
	gender: "male",
	// ...
}

let output = model.predict(input)
console.log(output)
```

```javascript
{ className: 'Negative', probability: 0.9381780624389648 }
```

### Inspect

Run `modelfox app`, open your browser to http://localhost:8080, and upload the model you trained.

- View stats and metrics.
- Tune your model to get the best performance.
- Make example predictions and get detailed explanations.

![report](./readme/report.png)

![tune](./readme/tune.png)

### Monitor

Once your model is deployed, make sure that it performs as well in production as it did in training. Opt in to logging by calling `logPrediction`.

```javascript
// Log the prediction.
model.logPrediction({
	identifier: "6c955d4f-be61-4ca7-bba9-8fe32d03f801",
	input,
	options,
	output,
})
```

Later on, if you find out the true value for a prediction, call `logTrueValue`.

```javascript
// Later on, if we get an official diagnosis for the patient, log the true value.
model.logTrueValue({
	identifier: "6c955d4f-be61-4ca7-bba9-8fe32d03f801",
	trueValue: "Positive",
})
```

Now you can:

- Look up any prediction by its identifier and get a detailed explanation.
- Get alerts if your data drifts or metrics dip.
- Track production accuracy, precision, recall, etc.

![predictions](./readme/predictions.png)

![drift](./readme/drift.png)

![metrics](./readme/metrics.png)

## Building from Source

This repository is a Cargo workspace, and does not require anything other than the latest nightly Rust toolchain to get started with.

1. Install [Rust](rust-lang.org) on Linux, macOS, or Windows.
2. Clone this repo and `cd` into it.
3. Run `cargo run` to run a debug build of the CLI.

If you are working on the app, run `scripts/app/dev`. This rebuilds and reruns the CLI with the `app` subcommand as you make changes.

To install all dependencies necessary to work on the language libraries and build releases, install [Nix](https://nixos.org) with [flake support](https://nixos.wiki/wiki/Flakes), then run `nix develop` or set up [direnv](https://github.com/direnv/direnv).

If you want to submit a pull request, please run `scripts/fmt` and `scripts/check` at the root of the repository to confirm that your changes are formatted correctly and do not have any errors.

## License

All of this repository is MIT licensed, except for the `crates/app` directory, which is source available and free to use for testing, but requires a paid license to use in production.
