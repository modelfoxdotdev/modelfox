<?php

declare(strict_types=1);

namespace modelfox\modelfox;

final class BagOfWordsFeatureContribution
{
    /**
     * This is the name of the source column for the feature group.
     */
    public string $column_name;
    /**
     * This is the ngram for the feature.
     */
    public Ngram $ngram;
    /**
     * This is the value the feature.
     */
    public float $feature_value;
    /**
     * This is the amount that the feature contributed to the output..
     */
    public float $feature_contribution_value;
    /**
     * Create a new BagOfWordsFeatureContribution instance
     * @param string $column_name
     * @param Unigram|Bigram $ngram
     * @param float $feature_value
     * @param float $feature_contribution_value
     * @return void
     */
    public function __construct(string $column_name, Ngram $ngram, float $feature_value, float $feature_contribution_value)
    {
        $this->column_name = $column_name;
        $this->ngram = $ngram;
        $this->feature_value = $feature_value;
        $this->feature_contribution_value = $feature_contribution_value;
    }
}
