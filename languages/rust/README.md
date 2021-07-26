# Tangram for Rust

- [Watch the Video](https://www.tangram.dev)
- [Read the Docs](https://www.tangram.dev/docs)

The Tangram Rust library makes it easy to make predictions with your Tangram machine learning model from Rust.

## Usage

```toml
[dependencies]
tangram = "*"
```

```rust
let model: tangram::Model = tangram::Model::from_path("heart_disease.tangram", None).unwrap();

let input = tangram::predict_input! {
  "age": 63.0,
  "gender": "male",
  // ...
};

let output = model.predict_one(input, None);
```

For more information, [read the docs](https://www.tangram.dev/docs).

## Examples

The source for this package contains a number of examples in the `examples` directory. Each example has a `README.md` explaining how to run it.
