# Tangram for Ruby

- [Watch the Video](https://www.tangram.xyz)
- [Read the Docs](https://www.tangram.xyz/docs)

The Tangram Ruby gem makes it easy to make predictions with your Tangram machine learning model from Ruby.

## Usage

```
$ gem install tangram
```

```ruby
require 'tangram'

model = Tangram::Model.from_file('./heart_disease.tangram')

input = {
  age: 63,
  gender: 'male',
  # ...
}

output = model.predict(input)
```

For more information, [read the docs](https://www.tangram.xyz/docs).

## Platform Support

Tangram for Ruby is currently supported on Linux, macOS, and Windows with AMD64 CPUs. Are you interested in another platform? [Open an issue](https://github.com/tangramxyz/tangram/issues/new) or send us an email at [help@tangram.xyz](mailto:help@tangram.xyz).

## Examples

The source for this gem contains two examples, `examples/predict` and `examples/monitor`.

### Basic

The basic example demonstrates loading a model from a `.tangram` file and making a prediction.

To run the example:

```
$ ruby examples/basic.rb
```

### Advanced

The advanced example demonstrates logging predictions and true values to the Tangram app. Before running the example, run `tangram app` to start the app running locally, open `http://localhost:8080` in your browser, and upload the file `examples/heart_disease.tangram` to it.

To run the example:

```
$ TANGRAM_URL=http://localhost:8080 ruby examples/advanced.rb
```

Now if you refresh the production stats or production metrics tabs for the model you uploaded, you should see predictions and true values.

For more information, [read the docs](https://www.tangram.xyz/docs).
