<?php

declare(strict_types=1);

namespace tangramdotdev\tangram;

final class Bigram
{
  /**
   * This is the first token in the bigram.
   */
  public string $token_a;
  /**
   * This is the second token in the bigram.
   */
  public string $token_b;
  /**
   * Create a new Biigram instance
   * @param string $token_a
   * @param string $token_b
   * @return void
   */
  public function __construct(string $token_a, string $token_b)
  {
    $this->token_a = $token_a;
    $this->token_b = $token_b;
  }
}
