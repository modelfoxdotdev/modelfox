
<?php

// TODO - detect platform, select proper library

FFI::load(__DIR__ . "/libtangram/x86_64-unknown-linux-gnu/tangram.h");
opcache_compile_file(__DIR__ . "src/LibTangram.php");

?>
