<?php

declare(strict_types=1);

namespace modelfox\modelfox;

final class FeatureContributions
{
    /**
     * This is the value the model would output if all features had baseline values.
     */
    public float $baseline;
    /**
     * This is the value the model output. Any difference from the `baseline_value` is because of the deviation of the features from their baseline values.
     */
    public float $output;
    /**
     * This list will contain one entry for each of the model's features. Note that features are computed from columns, so there will likely be more features than columns.
     */
    public array $entries;
    /**
     * Create a new FeatureContributions instance
     * @param float $baseline
     * @param float $output
     * @param array $entries
     * @return void
     */
    public function __construct(float $baseline, float $output, array $entries)
    {
        $this->baseline = $baseline;
        $this->output = $output;
        $this->entries = $entries;
    }
}
