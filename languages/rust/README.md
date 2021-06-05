# Tangram for Rust

- [Watch the Video](https://www.tangram.xyz)
- [Read the Docs](https://www.tangram.xyz/docs)

The Tangram Rust crate makes it easy to make predictions with your Tangram machine learning model from Rust.

## Usage

```toml
[dependencies]
tangram = { version = "0.4" }
```

```rust
let model = tangram::Model::<Input, Output>::from_path("examples/heart-disease.tangram");

let input = tangram::predict_input! {
  "age": 63,
  "gender": "male",
  // ...
};

let output = model.predict_one(input, None);
```

For more information, [read the docs](https://www.tangram.xyz/docs).

## Platform Support

Tangram for Rust is currently supported on Linux, macOS, and Windows with AMD64 CPUs. Are you interested in another platform? [Open an issue](https://github.com/tangramxyz/tangram/issues/new) or send us an email at [help@tangram.xyz](mailto:help@tangram.xyz).

## Examples

The source for this crate contains two examples, `examples/basic.rs` and `examples/advanced.rs`.

### Basic

The basic example demonstrates loading a model from a `.tangram` file and making a prediction.

To run the example:

```
$ cargo run --example basic
```

### Advanced

The advanced example demonstrates logging predictions and true values to the Tangram app. Before running the example, run `tangram app` to start the app running locally, open `http://localhost:8080` in your browser, and upload the file `examples/heart_disease.tangram` to it.

To run the example:

```
$ TANGRAM_URL=http://localhost:8080 cargo run --example advanced
```

Now if you refresh the production stats or production metrics tabs for the model you uploaded, you should see predictions and true values.

For more information, [read the docs](https://www.tangram.xyz/docs).
