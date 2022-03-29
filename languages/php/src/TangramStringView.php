<?php

declare(strict_types=1);

namespace modelfox\modelfox;

final class ModelFoxStringView
{
    /**
     * Pointer to C struct
     */
    private ?\FFI\CData $string_view = null;
    /**
     * Create a new LibModelFox instance
     * @param \FFI handle to FFI lib
     * @return void
     */
    public function __construct(\FFI $ffi)
    {
        $this->string_view = $ffi->new($ffi->type('modelfox_string_view'));
    }

    /**
     * Retrieve an unmanaged pointer to the inner stringview
     *
     * @return \FFI\CData raw modelfox_string_view pointer
     */
    public function raw_ptr()
    {
        return \FFI::addr($this->string_view);
    }

    /**
     * Retrieve the string referenced by this view
     *
     * @return string version string
     */
    public function into_string()
    {
        return substr($this->string_view->ptr, 0, $this->string_view->len);
    }
}
