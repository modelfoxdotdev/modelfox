<?php

declare(strict_types=1);

namespace modelfox\modelfox;

final class LoadModelOptions
{
    /**
     * If you are running the app locally or on your own server, use this field to provide the url to it. If not specified, the default value is https://app.modelfox.dev.
     * */
    public string $modelfox_url;

    /**
     * Create a new LoadModelOptions instance
     * @param string Custom URL to override default
     * @return void
     */
    public function __construct(string $modelfox_url)
    {
        $this->modelfox_url = $modelfox_url;
    }
}
