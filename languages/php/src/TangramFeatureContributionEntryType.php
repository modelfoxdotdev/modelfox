<?php

declare(strict_types=1);

namespace tangram\tangram;

abstract class TangramFeatureContributionEntryType
{
  const identity = 0;
  const normalized = 1;
  const one_hot_encoded = 2;
  const bag_of_words = 3;
  const bag_of_words_cosine_similarity = 4;
  const word_embedding = 5;
}
