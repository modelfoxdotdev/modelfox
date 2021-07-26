# Tangram for Ruby

- [Watch the Video](https://www.tangram.dev)
- [Read the Docs](https://www.tangram.dev/docs)

The Tangram Ruby library makes it easy to make predictions with your Tangram machine learning model from Ruby.

## Usage

```
$ gem install tangram
```

```ruby
require 'tangram'

model = Tangram::Model.from_path('./heart_disease.tangram')

input = {
  age: 63,
  gender: 'male',
  # ...
}

output = model.predict(input)
```

For more information, [read the docs](https://www.tangram.dev/docs).

## Platform Support

Tangram for Ruby is currently supported on Linux, macOS, and Windows with ARM64 and AMD64 CPUs. Are you interested in another platform? [Open an issue](https://github.com/tangramdotdev/tangram/issues/new) or send us an email at [help@tangram.dev](mailto:help@tangram.dev).

## Examples

The source for this package contains a number of examples in the `examples` directory. Each example has a `README.md` explaining how to run it.
