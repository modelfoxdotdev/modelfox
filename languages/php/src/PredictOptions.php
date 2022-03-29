<?php

declare(strict_types=1);

namespace modelfox\modelfox;

final class PredictOptions
{
    /**
     * If your model is a binary classifier, use this field to make predictions using a threshold chosen on the tuning page of the app. The default value is `0.5`.
     */
    public float $threshold;
    /**
     * Computing feature contributions is disabled by default. If you set this field to `true`, you will be able to access the feature contributions with the `feature_contributions` field of the predict output.
     */
    public bool $compute_feature_contributions;
    /**
     * Create a new PredictOptions instance
     * @param bool $compute_feature_contributions
     * @param float $threshold
     * @return void
     */
    public function __construct(bool $compute_feature_contributions, float $threshold = null)
    {
        $this->threshold = $threshold;
        $this->compute_feature_contributions = $compute_feature_contributions;
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
