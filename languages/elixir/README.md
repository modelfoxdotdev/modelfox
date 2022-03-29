# ModelFox for Elixir

- [Watch the Video](https://www.modelfox.dev)
- [Read the Docs](https://www.modelfox.dev/docs)

The ModelFox Elixir library makes it easy to make predictions with your ModelFox machine learning model from Elixir.

## Usage

Add the `modelfox` package to your `mix.exs`.

```elixir
model = ModelFox.load_model_from_path("heart_disease.modelfox")

input = %{
  :age => 63.0,
  :gender => "male",
  # ...
}

output = ModelFox.predict(model, input)
```

For more information, [read the docs](https://www.modelfox.dev/docs).

## Platform Support

ModelFox for Elixir is currently supported on Linux, macOS, and Windows with ARM64 and AMD64 CPUs. Are you interested in another platform? [Open an issue](https://github.com/modelfoxdotdev/modelfox/issues/new) or send us an email at [help@modelfox.dev](mailto:help@modelfox.dev).

## Examples

The source for this package contains a number of examples in the `examples` directory. Each example has a `README.md` explaining how to run it.
