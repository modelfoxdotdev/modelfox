<?php

declare(strict_types=1);

namespace tangram\tangram;

final class LoadModelOptions
{
    /**
     * If you are running the app locally or on your own server, use this field to provide the url to it. If not specified, the default value is https://app.tangram.dev.
     * */
    public string $tangram_url;

    /**
     * Create a new LoadModelOptions instance
     * @param string Custom URL to override default
     * @return void
     */
    public function __construct(string $tangram_url)
    {
        $this->tangram_url = $tangram_url;
    }
}
