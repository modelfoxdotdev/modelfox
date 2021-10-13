<?php

declare(strict_types=1);

namespace tangramdotdev\tangram;

abstract class TangramTaskType
{
  const regression = 0;
  const binary_classification = 1;
  const multiclass_classification = 2;
}
