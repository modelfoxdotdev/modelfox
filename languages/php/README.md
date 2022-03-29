# ModelFox for PHP

- [Watch the Video](https://www.modelfox.dev)
- [Read the Docs](https://www.modelfox.dev/docs)

The ModelFox PHP library makes it easy to make predictions with your ModelFox machine learning model from PHP.

## Usage

Add `modelfox/modelfox` to your `composer.json` with this command:

```json
$ composer require modelfox/modelfox
```

```php
<?php

namespace modelfox\modelfox;

require_once(dirname(dirname(__FILE__)) . '/vendor/autoload.php');

$model_path = dirname(dirname(__FILE__)) . '/heart_disease.modelfox';
$model = Model::from_path($model_path);

$input = [
    'age' => 63.0,
    'gender' => 'male',
    // ..
];

$output = $model->predict($input);
```

For more information, [read the docs](https://www.modelfox.dev/docs).

## Platform Support

ModelFox for PHP is currently supported on Linux, macOS, and Windows with ARM64 and AMD64 CPUs. Are you interested in another platform? [Open an issue](https://github.com/modelfoxdotdev/modelfox/issues/new) or send us an email at [help@modelfox.dev](mailto:help@modelfox.dev).

## Examples

The source for this package contains a number of examples in the `examples` directory. Each example has a `README.md` explaining how to run it.
