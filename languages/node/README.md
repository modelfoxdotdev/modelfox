# Tangram + Node.js

- [Watch the Video](https://www.tangram.xyz)
- [Read the Docs](https://www.tangram.xyz/docs)

The Tangram Node.js package makes it easy to make predictions with your Tangram machine learning model from Node.js.

## Usage

```
$ npm install @tangramxyz/tangram-node
```

```javascript
let tangram = require("@tangramxyz/tangram-node")

let model = new tangram.Model("./heart_disease.tangram")

let input = {
	age: 63,
	gender: "male",
	// ...
}

let output = model.predictSync(input)
```

For more information, [read the docs](https://www.tangram.xyz/docs).

## Platform Support

Tangram for Node is currently supported on Linux, macOS, and Windows with ARM64 and AMD64 CPUs. Are you interested in another platform? [Open an issue](https://github.com/tangramxyz/tangram/issues/new) or send us an email at [help@tangram.xyz](mailto:help@tangram.xyz).

## Examples

The source for this package contains three examples, `examples/basic.js`, `examples/advanced.js`, and `examples/typescript.ts`.

### Basic

The basic example demonstrates loading a model from a `.tangram` file and making a prediction.

To run the example:

```
$ npm run build
$ node examples/basic.js
```

### Advanced

The advanced example demonstrates logging predictions and true values to the Tangram app. Before running the example, run `tangram app` to start the app running locally, open `http://localhost:8080` in your browser, and upload the file `examples/heart_disease.tangram` to it.

To run the example:

```
$ npm run build
$ node examples/advanced.js
```

Now if you refresh the production stats or production metrics tabs for the model you uploaded, you should see predictions and true values.

For more information, [read the docs](https://www.tangram.xyz/docs).

### Typescript

The typescript example shows how to make predictions with your Tangram machine learning model in an app written in [TypeScript](https://www.typescriptlang.org). It demonstrates how to provide types for the input and output of a model.

To build and run the example:

```
$ npm run build
$ npx ts-node examples/typescript.ts
```
