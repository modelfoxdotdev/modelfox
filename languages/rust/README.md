# Tangram for Rust

- [Watch the Video](https://www.tangram.xyz)
- [Read the Docs](https://www.tangram.xyz/docs)

The Tangram Rust library makes it easy to make predictions with your Tangram machine learning model from Rust.

## Usage

```toml
[dependencies]
tangram = "*"
```

```rust
let model: tangram::Model = tangram::Model::from_path("examples/heart-disease.tangram", None).unwrap();

let input = tangram::predict_input! {
  "age": 63.0,
  "gender": "male",
  // ...
};

let output = model.predict_one(input, None);
```

For more information, [read the docs](https://www.tangram.xyz/docs).

## Platform Support

Tangram for Rust is currently supported on Linux, macOS, and Windows with ARM64 and AMD64 CPUs. Are you interested in another platform? [Open an issue](https://github.com/tangramxyz/tangram/issues/new) or send us an email at [help@tangram.xyz](mailto:help@tangram.xyz).

## Examples

The source for this package contains a number of examples in the `examples` directory. Each example has a `README.md` explaining how to run it.
