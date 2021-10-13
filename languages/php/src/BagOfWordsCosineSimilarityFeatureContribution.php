<?php

declare(strict_types=1);

namespace tangram\tangram;

final class BagOfWordsCosineSimilarityFeatureContribution
{
    /**
     * This is the name of the source column a for the feature group.
     */
    public string $column_name_a;
    /**
     * This is the name of the source column b for the feature group.
     */
    public string $column_name_b;
    /**
     * This is the value the feature.
     */
    public float $feature_value;
    /**
     * This is the amount that the feature contributed to the output..
     */
    public float $feature_contribution_value;
    /**
     * Create a new BagOfWordsCosineSimilarityFeatureContribution instance
     * @param string $column_name_a
     * @param string $column_name_b
     * @param float $feature_value
     * @param float $feature_contribution_value
     * @return void
     */
    public function __construct(string $column_name_a, string $column_name_b, float $feature_value, float $feature_contribution_value)
    {
        $this->column_name_a = $column_name_a;
        $this->column_name_b = $column_name_b;
        $this->feature_value = $feature_value;
        $this->feature_contribution_value = $feature_contribution_value;
    }
}
