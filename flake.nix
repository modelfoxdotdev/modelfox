{
  inputs = {
    nixpkgs = {
      url = "github:nixos/nixpkgs/nixos-unstable";
    };
    flake-utils = {
      url = "github:numtide/flake-utils";
    };
    fenix = {
      url = "github:nix-community/fenix";
    };
    windows_sdk = {
      url = "github:tangramdotdev/windows_sdk";
    };
  };
  outputs = { fenix, flake-utils, nixpkgs, windows_sdk, ... }: flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
      };
      rust = fenix.packages.${system}.fromToolchainFile { 
        file = ./rust-toolchain.toml;
        sha256 = "sha256-J+uisSFON0GwVfyFemT7Oe28ziaZMelA+PgqJB2A4aw=";
      };
    in rec {
      devShell = pkgs.mkShell {
        buildInputs = with pkgs; [
          cachix
          cargo-insta
          clang_13
          createrepo_c
          doxygen
          dpkg
          elixir
          go
          lld_13
	  llvm_13
          mold
          nodejs-16_x
          (php.withExtensions ({ all, ...}: with all; [
            curl
            dom
            ffi
            fileinfo
            filter
            iconv
            mbstring
            simplexml
            tokenizer
          ]))
          php.packages.composer
          (python39.withPackages(ps: with ps; [
            catboost
            lightgbm
            numpy
            pandas
            pytorch
            scikitlearn
            xgboost
          ]))
          rpm
          ruby
          rust
          rust-cbindgen
          sequoia
          sqlite
          time
          wasm-bindgen-cli
          windows_sdk
          (zig.overrideAttrs (old: {
            src = pkgs.fetchFromGitHub {
              owner = "ziglang";
              repo = "zig";
              rev = "002fbb0af043d90b0ab7d2f2804effc6fa2d690c";
              hash = "sha256-a4IXh4gfv34exfLPqxcS+7e3bOqL1AJNWzBMXm2tTvU=";
            };
            patches = [ ./zig_x86_64-windows-gnu_static_library_search.patch ];
            nativeBuildInputs = [
              cmake
              llvmPackages_13.llvm.dev
            ];
            buildInputs = [
              libxml2
              zlib
            ] ++ (with llvmPackages_13; [
              libclang
              lld
              llvm
            ]);
          }))
        ];

        CARGO_UNSTABLE_HOST_CONFIG = "true";
        CARGO_UNSTABLE_TARGET_APPLIES_TO_HOST = "true";

        CARGO_TARGET_APPLIES_TO_HOST = "false";

        # x86_64-unknown-linux-gnu
        CARGO_HOST_X86_64_UNKNOWN_LINUX_GNU_LINKER = "${pkgs.writeShellScriptBin "linker" ''
          clang -fuse-ld=$(which mold) $@
        ''}/bin/linker";

        # aarch64-linux-gnu_2_28
        CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = "${pkgs.writeShellScriptBin "linker" ''
          zig cc -target aarch64-linux-gnu.2.28 $@
        ''}/bin/linker";
        CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS = "-C target-feature=-outline-atomics";
        CC_aarch64_unknown_linux_gnu = "${pkgs.writeShellScriptBin "cc" ''
          zig cc -target aarch64-linux-gnu.2.28 $@
        ''}/bin/cc";

        # aarch64-linux-musl
        CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER = "${pkgs.writeShellScriptBin "linker" ''
          zig cc -target aarch64-linux-musl -lc -dynamic $@
        ''}/bin/linker";
        CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUSTFLAGS = "-C target-feature=-outline-atomics";
        CC_aarch64_unknown_linux_musl = "${pkgs.writeShellScriptBin "cc" ''
          zig cc -target aarch64-linux-musl $@
        ''}/bin/cc";

        # aarch64-macos
        CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER = "${pkgs.writeShellScriptBin "linker" ''
          zig cc -target aarch64-macos $@
        ''}/bin/linker";
        CC_aarch64_apple_darwin = "${pkgs.writeShellScriptBin "cc" ''
          zig cc -target aarch64-macos $@
        ''}/bin/cc";

        # wasm32
        CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER = "lld";

        # x86_64-linux-gnu_2_28
        CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_APPLIES_TO_HOST = "false";
        CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER = "${pkgs.writeShellScriptBin "linker" ''
          zig cc -target x86_64-linux-gnu.2.28 $@
        ''}/bin/linker";
        CC_x86_64_unknown_linux_gnu = "${pkgs.writeShellScriptBin "cc" ''
          zig cc -target x86_64-linux-gnu.2.28 $@
        ''}/bin/cc";

        # x86_64-linux-musl
        CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER = "${pkgs.writeShellScriptBin "linker" ''
          zig cc -target x86_64-linux-musl -lc -dynamic $@
        ''}/bin/linker";
        CC_x86_64_unknown_linux_musl = "${pkgs.writeShellScriptBin "cc" ''
          zig cc -target x86_64-linux-musl $@
        ''}/bin/cc";

        # x86_64-macos
        CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER = "${pkgs.writeShellScriptBin "linker" ''
          zig cc -target x86_64-macos $@
        ''}/bin/linker";
        CC_x86_64_apple_darwin = "${pkgs.writeShellScriptBin "cc" ''
          zig cc -target x86_64-macos $@
        ''}/bin/cc";

        # x86_64-windows-gnu
        CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER = "${pkgs.writeShellScriptBin "linker" ''
          for arg do
            shift
            [ "$arg" = "-lgcc" ] && continue
            [ "$arg" = "-lgcc_s" ] && continue
            [ "$arg" = "-lgcc_eh" ] && continue
            [ "$arg" = "-l:libpthread.a" ] && continue
            set -- "$@" "$arg"
          done
          zig cc -target x86_64-windows-gnu -lstdc++ $@
        ''}/bin/linker";
        CC_x86_64_pc_windows_gnu = "${pkgs.writeShellScriptBin "cc" ''
          zig cc -target x86_64-windows-gnu $@
        ''}/bin/cc";

        # x86_64-windows-msvc
        AR_x86_64_pc_windows_msvc = "llvm-lib";
        CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER = "${pkgs.writeShellScriptBin "linker" ''
          lld-link /libpath:$WINDOWS_SDK/clang/lib/x64 /libpath:$WINDOWS_SDK/crt/lib/x64 /libpath:$WINDOWS_SDK/sdk/lib/x64 /libpath:$WINDOWS_SDK/sdk/lib/x64/ucrt /libpath:$WINDOWS_SDK/sdk/lib/x64/um $@
        ''}/bin/linker";
        CC_x86_64_pc_windows_msvc = "${pkgs.writeShellScriptBin "cc" ''
          clang-cl /I $WINDOWS_SDK/clang/include /I $WINDOWS_SDK/crt/include /I $WINDOWS_SDK/sdk/include /I $WINDOWS_SDK/sdk/include/shared /I $WINDOWS_SDK/sdk/include/ucrt /I $WINDOWS_SDK/sdk/include/um $@
        ''}/bin/cc";
        WINDOWS_SDK = windows_sdk.defaultPackage.${system};
      };
    }
  );
}
