<?php

declare(strict_types=1);

namespace modelfox\modelfox;

final class RegressionPredictOutput extends PredictOutput
{
    /**
     * This is the predicted value.
     */
    public float $value;
    /**
     *  If computing feature contributions was enabled in the predict options, this value will explain the model's output, showing how much each feature contributed to the output.
     */
    public ?FeatureContributions $feature_contributions;
    /**
     * Create a new PredictOptions instance
     * @param float $value
     * @param ?FeatureContributions $feature_contributions
     * @return void
     */
    public function __construct(float $value, ?FeatureContributions $feature_contributions = null)
    {
        $this->value = $value;
        $this->feature_contributions = $feature_contributions;
    }

    /**
     * Serialize to JSON
     * @return string JSON representation
     */
    public function to_json()
    {
        return json_encode($this);
    }
}
