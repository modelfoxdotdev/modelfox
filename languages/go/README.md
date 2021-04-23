# Tangram for Go

- [Watch the Video](https://www.tangram.xyz)
- [Read the Docs](https://www.tangram.xyz/docs)

The Tangram Go module makes it easy to make predictions with your Tangram machine learning model from Go.

## Usage

```
$ go get -u github.com/tangramxyz/tangram-go
```

```go
import "github.com/tangramxyz/tangram-go"

model, _ := tangram.LoadModelFromPath("./heart_disease.tangram", nil)
defer model.Destroy()

input := tangram.PredictInput{
  "age":    63,
  "gender": "male",
  // ...
}

output := model.PredictOne(input, nil)

fmt.Println("Output:", output.ClassName)
```

For more information, [read the docs](https://www.tangram.xyz/docs).

## Platform Support

Tangram for Go is currently supported on the following combinations of `$GOARCH` and `$GOOS`:

- `amd64` `linux`
- `amd64` `darwin`
- `amd64` `windows`

Are you interested in another platform? [Open an issue](https://github.com/tangramxyz/tangram/issues/new) or send us an email at [help@tangram.xyz](mailto:help@tangram.xyz).

Tangram for Go links to the tangram C library, so cgo is required. The tangram C library will be linked statically into your executable, so when you run `go build` you will still get a statically linked executable you can run anywhere without having to worry about dynamic linking errors.

## Examples

This source for this module contains two examples, `examples/predict` and `examples/monitor`.

### Basic

The basic example demonstrates loading a model from a `.tangram` file and making a prediction.

```
$ go run examples/basic.go
```

### Advanced

The advanced example demonstrates logging predictions and true values to the Tangram app. Before running the example, run `tangram app` to start the app running locally, open `http://localhost:8080` in your browser, and upload the file `examples/heart_disease.tangram` to it.

To run the example:

```
$ TANGRAM_URL=http://localhost:8080 go run examples/advanced.go
```

Now if you refresh the production stats or production metrics tabs for the model you uploaded, you should see predictions and true values.

For more information, [read the docs](https://www.tangram.xyz/docs).
