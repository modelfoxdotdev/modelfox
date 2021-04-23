# Tangram for Python

- [Watch the Video](https://www.tangram.xyz)
- [Read the Docs](https://www.tangram.xyz/docs)

The Tangram Python package makes it easy to make predictions with your Tangram machine learning model from Python.

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

For more information, [read the docs](https://www.tangram.xyz/docs).

## Platform Support

Tangram for Python is currently supported on Linux, macOS, and Windows with AMD64 CPUs. Are you interested in another platform? [Open an issue](https://github.com/tangramxyz/tangram/issues/new) or send us an email at [help@tangram.xyz](mailto:help@tangram.xyz).

## Examples

The source for this package contains two examples, `examples/basic.py` and `examples/advanced.py`.

### Basic

The basic example demonstrates loading a model from a `.tangram` file and making a prediction.

To run the example:

```
$ python3 examples/basic.py
```

### Advanced

The advanced example demonstrates logging predictions and true values to the Tangram app. Before running the example, run `tangram app` to start the app running locally, open `http://localhost:8080` in your browser, and upload the file `examples/heart_disease.tangram` to it.

To run the example:

```
$ TANGRAM_URL=http://localhost:8080 python3 examples/advanced.py
```

Now if you refresh the production stats or production metrics tabs for the model you uploaded, you should see predictions and true values.

For more information, [read the docs](https://www.tangram.xyz/docs).
