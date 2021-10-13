<?php

declare(strict_types=1);

namespace tangram\tangram;

abstract class TangramTaskType
{
    public const regression = 0;
    public const binary_classification = 1;
    public const multiclass_classification = 2;
}
