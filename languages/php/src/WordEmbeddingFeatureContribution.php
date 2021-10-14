<?php

declare(strict_types=1);

namespace tangram\tangram;

final class WordEmbeddingFeatureContribution
{
    /**
     * This is the name of the source column for the feature group.
     */
    public string $column_name;
    /**
     * This is the index of the feature in the word embedding.
     */
    public int $value_index;
    /**
     * This is the amount that the feature contributed to the output..
     */
    public float $feature_contribution_value;
    /**
     * Create a new BagOfWordsCosineSimilarityFeatureContribution instance
     * @param string $column_name
     * @param int $value_index
     * @param float $feature_contribution_value
     * @return void
     */
    public function __construct(string $column_name, int $value_index, float $feature_contribution_value)
    {
        $this->column_name = $column_name;
        $this->value_index = $value_index;
        $this->feature_contribution_value = $feature_contribution_value;
    }
}
