<?php

return PhpCsFixer\Config::create()
  ->setRiskyAllowed(true)
  ->setRules([
    // Rulesets
    '@PSR2' => true,
    '@PhpCsFixer' => true,
    '@PhpCsFixer:risky' => true,
    '@PHP56Migration:risky' => true,
    '@PHPUnit57Migration:risky' => true,

    // Additional rules
    'fopen_flags' => true,
    'linebreak_after_opening_tag' => true,
    'native_constant_invocation' => true,
    'native_function_invocation' => true,
  ]);
