# ModelFox for Go

- [Watch the Video](https://www.modelfox.dev)
- [Read the Docs](https://www.modelfox.dev/docs)

The ModelFox Go module makes it easy to make predictions with your ModelFox machine learning model from Go.

## Usage

```
$ go get -u github.com/modelfoxdotdev/modelfox-go
```

```go
import "github.com/modelfoxdotdev/modelfox-go"

model, _ := modelfox.LoadModelFromPath("./heart_disease.modelfox", nil)
defer model.Destroy()

input := modelfox.PredictInput{
  "age":    63,
  "gender": "male",
  // ...
}

output := model.PredictOne(input, nil)

fmt.Println("Output:", output.ClassName)
```

For more information, [read the docs](https://www.modelfox.dev/docs).

## Platform Support

ModelFox for Go is currently supported on the following combinations of `$GOARCH` and `$GOOS`:

- `amd64` `linux`
- `arm64` `linux`
- `amd64` `darwin`
- `arm64` `darwin`
- `amd64` `windows`

Are you interested in another platform? [Open an issue](https://github.com/modelfoxdotdev/modelfox/issues/new) or send us an email at [help@modelfox.dev](mailto:help@modelfox.dev).

ModelFox for Go links to the modelfox C library, so cgo is required. The modelfox C library will be linked statically into your executable, so when you run `go build` you will still get a statically linked executable you can run anywhere without having to worry about dynamic linking errors.

## Examples

The source for this package contains a number of examples in the `examples` directory. Each example has a `README.md` explaining how to run it.
