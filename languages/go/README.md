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
- `arm64` `linux`
- `amd64` `darwin`
- `arm64` `darwin`
- `amd64` `windows`

Are you interested in another platform? [Open an issue](https://github.com/tangramxyz/tangram/issues/new) or send us an email at [help@tangram.xyz](mailto:help@tangram.xyz).

Tangram for Go links to the tangram C library, so cgo is required. The tangram C library will be linked statically into your executable, so when you run `go build` you will still get a statically linked executable you can run anywhere without having to worry about dynamic linking errors.

## Examples

The source for this package contains a number of examples in the `examples` directory. Each example has a `README.md` explaining how to run it.
