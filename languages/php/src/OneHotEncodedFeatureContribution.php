<?php

declare(strict_types=1);

namespace tangramdotdev\tangram;

final class OneHotEncodedFeatureContribution
{
  /**
   * This is the name of the source column for the feature group.
   */
  public string $column_name;
  /**
   * This is the enum variant tha the feature indicates the presence of.
   */
  public string|null $variant;
  /**
   * This is the value the feature.
   */
  public float $feature_value;
  /**
   * This is the amount that the feature contributed to the output..
   */
  public float $feature_contribution_value;
  /**
   * Create a new OneHotEncodedFeatureContribution instance
   * @param string $column_name
   * @param string|null $variant
   * @param float $feature_value
   * @param float $feature_contribution_value
   * @return void
   */
  public function __construct(string $column_name, string|null $variant, float $feature_value, float $feature_contribution_value)
  {
    $this->column_name = $column_name;
    $this->variant = $variant;
    $this->feature_value = $feature_value;
    $this->feature_contribution_value = $feature_contribution_value;
  }
}
