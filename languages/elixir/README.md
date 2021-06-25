# Tangram for Elixir

- [Watch the Video](https://www.tangram.xyz)
- [Read the Docs](https://www.tangram.xyz/docs)

The Tangram Elixir package makes it easy to make predictions with your Tangram machine learning model from Elixir.

## Usage

Add the `tangram` package to your `mix.exs`.

```elixir
model = Tangram.load_model_from_path("heart_disease.tangram")

input = %{
  :age => 63.0,
  :gender => "male",
  # ...
}

output = Tangram.predict(model, input)
```

For more information, [read the docs](https://www.tangram.xyz/docs).

## Platform Support

Tangram for Elixir is currently supported on Linux, macOS, and Windows with ARM64 and AMD64 CPUs. Are you interested in another platform? [Open an issue](https://github.com/tangramxyz/tangram/issues/new) or send us an email at [help@tangram.xyz](mailto:help@tangram.xyz).

## Examples

The source for this package contains two examples, `examples/basic.exs` and `examples/advanced.exs`.

### Basic

The basic example demonstrates loading a model from a `.tangram` file and making a prediction.

To run the example:

```
$ mix run examples/basic.exs
```

### Advanced

The advanced example demonstrates logging predictions and true values to the Tangram app. Before running the example, run `tangram app` to start the app running locally, open `http://localhost:8080` in your browser, and upload the file `examples/heart_disease.tangram` to it.

To run the example:

```
$ TANGRAM_URL=http://localhost:8080 mix run examples/advanced.exs
```

Now if you refresh the production stats or production metrics tabs for the model you uploaded, you should see predictions and true values.

For more information, [read the docs](https://www.tangram.xyz/docs).

### Notes

- On Alpine Linux, Tangram for Elixir requires the `libgcc` library to be installed. It is not installed by default in the Alpine Linux docker image, but will very likely be a dependency of software you are already using. If not, you can run `apk add libgcc` to install it. We have opened [this issue](https://github.com/rust-lang/rust/issues/82521) with Rust to hopefully eliminate this requirement in the future.
