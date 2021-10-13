# Tangram for PHP

- [Watch the Video](https://www.tangram.dev)
- [Read the Docs](https://www.tangram.dev/docs)

The Tangram PHP library makes it easy to make predictions with your Tangram machine learning model from PHP.

## Usage

Add `tangramdotdev/tangram` to your `composer.json`:

```json
	"require": {
		"tangramdotdev/tangram": "0.7.0"
	},
```

```php
<?php

namespace tangramdotdev\tangram;

require dirname(dirname(__FILE__)) . '/vendor/autoload.php';

$model_path = dirname(dirname(__FILE__)) . '/heart_disease.tangram';
$model = Model::from_path($model_path);

$input = [
    'age' => 63.0,
    'gender' => 'male',
    // ..
];

$output = $model->predict($input)
```

For more information, [read the docs](https://www.tangram.dev/docs).

## Platform Support

Tangram for PHP is currently supported on Linux, macOS, and Windows with ARM64 and AMD64 CPUs. Are you interested in another platform? [Open an issue](https://github.com/tangramdotdev/tangram/issues/new) or send us an email at [help@tangram.dev](mailto:help@tangram.dev).

## Examples

The source for this package contains a number of examples in the `examples` directory. Each example has a `README.md` explaining how to run it.
