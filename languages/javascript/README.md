# ModelFox for JavaScript

- [Watch the Video](https://www.modelfox.dev)
- [Read the Docs](https://www.modelfox.dev/docs)

The ModelFox JavaScript package makes it easy to make predictions with your ModelFox machine learning model from JavaScript.

## Setup

### Node.js

```
$ npm install @modelfoxdotdev/modelfox
```

```javascript
let modelfox = require("@modelfoxdotdev/modelfox")
```

### Deno

```javascript
import modelfox from "https://js.modelfox.dev/deno"
```

### Bundler

If you are using a bundler that supports WebAssembly modules such as Webpack >= 4, you can use the package from npm.

```
$ npm install @modelfoxdotdev/modelfox
```

```javascript
import modelfox from "@modelfoxdotdev/modelfox"
```

### Browser

If you are targeting a modern browser with support for ES Modules, WebAssembly, and top-level await, you can import the modelfox library from https://js.modelfox.dev.

```javascript
import modelfox from "https://js.modelfox.dev"
```

## Usage

```javascript
let model = new modelfox.Model("./heart_disease.modelfox")

let input = {
	age: 63,
	gender: "male",
	// ...
}

let output = model.predict(input)
```

For more information, [read the docs](https://www.modelfox.dev/docs).

## Examples

The source for this package contains a number of examples in the `examples` directory. Each example has a `README.md` explaining how to run it.
