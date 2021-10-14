<?php

declare(strict_types=1);

namespace tangram\tangram;

final class Unigram extends Ngram
{
    /**
     * This is the token.
     */
    public string $token;
    /**
     * Create a new Unigram instance
     * @param string $token
     * @return void
     */
    public function __construct(string $token)
    {
        $this->token = $token;
    }
}
