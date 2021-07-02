# Tangram for JavaScript

- [Watch the Video](https://www.tangram.xyz)
- [Read the Docs](https://www.tangram.xyz/docs)

The Tangram JavaScript package makes it easy to make predictions with your Tangram machine learning model from JavaScript.

## Setup

### Node.js

```
$ npm install @tangramxyz/tangram
```

```javascript
let tangram = require("@tangramxyz/tangram")
```

### Deno

_not yet implemented_

```javascript
import tangram from "https://deno.land/x/tangram"
```

### Bundler

If you are using a bundler that supports WebAssembly modules such as Webpack >= 4, you can use the package from npm.

```
$ npm install @tangramxyz/tangram
```

```javascript
import tangram from "@tangramxyz/tangram"
```

### ES Modules

_not yet implemented_

If you are targeting a modern browser with support for ES Modules, WebAssembly, and top-level await, you can import the tangram library from https://js.tangram.xyz.

```javascript
import tangram from "https://js.tangram.xyz"
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

For more information, [read the docs](https://www.tangram.xyz/docs).

## Examples

The source for this package contains a number of examples in the `examples` directory. Each example has a `README.md` explaining how to run it.
