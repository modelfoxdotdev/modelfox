# Tangram for Python

- [Watch the Video](https://www.tangram.dev)
- [Read the Docs](https://www.tangram.dev/docs)

The Tangram Python library makes it easy to make predictions with your Tangram machine learning model from Python.

## Usage

```
$ pip install tangram
```

```python
import tangram

model = tangram.Model.from_path('./heart_disease.tangram')

input = {
  'age': 63,
  'gender': 'male',
  # ...
}

output = model.predict(input)
```

For more information, [read the docs](https://www.tangram.dev/docs).

## Platform Support

Tangram for Python is currently supported on Linux, macOS, and Windows with ARM64 and AMD64 CPUs. Are you interested in another platform? [Open an issue](https://github.com/tangramdotdev/tangram/issues/new) or send us an email at [help@tangram.dev](mailto:help@tangram.dev).

## Examples

The source for this package contains a number of examples in the `examples` directory. Each example has a `README.md` explaining how to run it.

## Notes

- Tangram for Python requires pip 20.3 or later, which includes support for [PEP 600](https://www.python.org/dev/peps/pep-0600/). This is necessary to correctly install tangram's native extension. If you are using an earlier python version, you may see the error "Could not find a version that satisfies the requirement tangram" when you attempt to `pip install tangram`. To resolve this, you should upgrade pip to version 20.3 or later.
