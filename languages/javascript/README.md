# Tangram for JavaScript

- [Watch the Video](https://www.tangram.dev)
- [Read the Docs](https://www.tangram.dev/docs)

The Tangram JavaScript package makes it easy to make predictions with your Tangram machine learning model from JavaScript.

## Setup

### Node.js

```
$ npm install @tangramdotdev/tangram
```

```javascript
let tangram = require("@tangramdotdev/tangram")
```

### Deno

```javascript
import tangram from "https://js.tangram.dev/deno"
```

### Bundler

If you are using a bundler that supports WebAssembly modules such as Webpack >= 4, you can use the package from npm.

```
$ npm install @tangramdotdev/tangram
```

```javascript
import tangram from "@tangramdotdev/tangram"
```

### Browser

If you are targeting a modern browser with support for ES Modules, WebAssembly, and top-level await, you can import the tangram library from https://js.tangram.dev.

```javascript
import tangram from "https://js.tangram.dev"
```

## Usage

```javascript
let model = new tangram.Model("./heart_disease.tangram")

let input = {
	age: 63,
	gender: "male",
	// ...
}

let output = model.predict(input)
```

For more information, [read the docs](https://www.tangram.dev/docs).

## Examples

The source for this package contains a number of examples in the `examples` directory. Each example has a `README.md` explaining how to run it.
