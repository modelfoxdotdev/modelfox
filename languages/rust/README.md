# ModelFox for Rust

- [Watch the Video](https://www.modelfox.dev)
- [Read the Docs](https://www.modelfox.dev/docs)

The ModelFox Rust library makes it easy to make predictions with your ModelFox machine learning model from Rust.

## Usage

```toml
[dependencies]
modelfox = { git = "https://github.com/modelfoxdotdev/modelfox" }
```

```rust
let model: modelfox::Model = modelfox::Model::from_path("heart_disease.modelfox", None).unwrap();

let input = modelfox::predict_input! {
  "age": 63.0,
  "gender": "male",
  // ...
};

let output = model.predict_one(input, None);
```

For more information, [read the docs](https://www.modelfox.dev/docs).

## Examples

The source for this package contains a number of examples in the `examples` directory. Each example has a `README.md` explaining how to run it.
