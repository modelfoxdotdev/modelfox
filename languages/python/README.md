# ModelFox for Python

- [Watch the Video](https://www.modelfox.dev)
- [Read the Docs](https://www.modelfox.dev/docs)

The ModelFox Python library makes it easy to make predictions with your ModelFox machine learning model from Python.

## Usage

```
$ pip install modelfox
```

```python
import modelfox

model = modelfox.Model.from_path('./heart_disease.modelfox')

input = {
  'age': 63,
  'gender': 'male',
  # ...
}

output = model.predict(input)
```

For more information, [read the docs](https://www.modelfox.dev/docs).

## Platform Support

ModelFox for Python is currently supported on Linux, macOS, and Windows with ARM64 and AMD64 CPUs. Are you interested in another platform? [Open an issue](https://github.com/modelfoxdotdev/modelfox/issues/new) or send us an email at [help@modelfox.dev](mailto:help@modelfox.dev).

## Examples

The source for this package contains a number of examples in the `examples` directory. Each example has a `README.md` explaining how to run it.

## Notes

- ModelFox for Python requires pip 20.3 or later, which includes support for [PEP 600](https://www.python.org/dev/peps/pep-0600/). This is necessary to correctly install modelfox's native extension. If you are using an earlier python version, you may see the error "Could not find a version that satisfies the requirement modelfox" when you attempt to `pip install modelfox`. To resolve this, you should upgrade pip to version 20.3 or later.
