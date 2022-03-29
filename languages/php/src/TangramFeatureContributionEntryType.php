<?php

declare(strict_types=1);

namespace modelfox\modelfox;

abstract class ModelFoxFeatureContributionEntryType
{
    public const identity = 0;
    public const normalized = 1;
    public const one_hot_encoded = 2;
    public const bag_of_words = 3;
    public const bag_of_words_cosine_similarity = 4;
    public const word_embedding = 5;
}
