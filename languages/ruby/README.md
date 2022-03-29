# ModelFox for Ruby

- [Watch the Video](https://www.modelfox.dev)
- [Read the Docs](https://www.modelfox.dev/docs)

The ModelFox Ruby library makes it easy to make predictions with your ModelFox machine learning model from Ruby.

## Usage

```
$ gem install modelfox
```

```ruby
require 'modelfox'

model = ModelFox::Model.from_path('./heart_disease.modelfox')

input = {
  age: 63,
  gender: 'male',
  # ...
}

output = model.predict(input)
```

For more information, [read the docs](https://www.modelfox.dev/docs).

## Platform Support

ModelFox for Ruby is currently supported on Linux, macOS, and Windows with ARM64 and AMD64 CPUs. Are you interested in another platform? [Open an issue](https://github.com/modelfoxdotdev/modelfox/issues/new) or send us an email at [help@modelfox.dev](mailto:help@modelfox.dev).

## Examples

The source for this package contains a number of examples in the `examples` directory. Each example has a `README.md` explaining how to run it.
