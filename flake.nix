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
    wheel_writer = {
      url = "github:tangramdotdev/wheel_writer";
    };
    windows_sdk = {
      url = "github:tangramdotdev/windows_sdk";
    };
  };
  outputs = {
    fenix,
    flake-utils,
    nixpkgs,
    wheel_writer,
    windows_sdk,
    ...
  }: flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          (self: super: {
            rpm = super.rpm.overrideAttrs (_: {
              patches = [
                (pkgs.fetchpatch {
                  url = "https://github.com/rpm-software-management/rpm/pull/1775.patch";
                  sha256 = "0zzblwx9apxyjsri4cxd09y9b2hs57r2fck98939j1qgcwy732ar";
                })
              ];
            });
            rust-cbindgen = super.rust-cbindgen.overrideAttrs (_: {
              doCheck = false;
            });
            zig = super.zig.overrideAttrs (_: {
              src = self.fetchFromGitHub {
                owner = "ziglang";
                repo = "zig";
                rev = "adf059f272dfd3c1652bce774c0b6c204d5d6b8b";
                hash = "sha256-pnNfvdLBN8GrVHz+Cf5QX3VHC+s3jjNO/vtzgGD132Y=";
              };
              patches = [
                (self.fetchpatch {
                  url = "https://github.com/ziglang/zig/pull/9771.patch";
                  sha256 = "sha256-AaMNNBET/x0f3a9oxpgBZXnUdKH4bydKMLJfXLBmvZo=";
                })
              ];
              nativeBuildInputs = with self; [
                cmake
                llvmPackages_13.llvm.dev
              ];
              buildInputs = with self; [
                libxml2
                zlib
              ] ++ (with llvmPackages_13; [
                libclang
                lld
                llvm
              ]);
            });
          })
        ];
      };
      rust =
        let
          toolchain = {
            channel = "nightly";
            date = "2021-12-20";
            sha256 = "sha256-FTlFODbchSsFDRGVTd6HkY5QeeZ2YgFV9HCubYl6TJQ=";
          };
        in with fenix.packages.${system}; combine (with toolchainOf toolchain; [
          cargo
          clippy-preview
          rust-src
          rust-std
          rustc
          rustfmt-preview
          (targets.aarch64-unknown-linux-gnu.toolchainOf toolchain).rust-std
          (targets.aarch64-unknown-linux-musl.toolchainOf toolchain).rust-std
          (targets.aarch64-apple-darwin.toolchainOf toolchain).rust-std
          (targets.wasm32-unknown-unknown.toolchainOf toolchain).rust-std
          (targets.x86_64-unknown-linux-gnu.toolchainOf toolchain).rust-std
          (targets.x86_64-unknown-linux-musl.toolchainOf toolchain).rust-std
          (targets.x86_64-apple-darwin.toolchainOf toolchain).rust-std
          (targets.x86_64-pc-windows-gnu.toolchainOf toolchain).rust-std
          (targets.x86_64-pc-windows-msvc.toolchainOf toolchain).rust-std
        ]);
    in {
      devShell = pkgs.mkShell {
        packages = with pkgs; [
          cachix
          cargo-insta
          cargo-outdated
          clang_13
          createrepo_c
          doxygen
          dpkg
          elixir
          gnupg
          go
          libiconv
          lld_13
          llvm_13
          mold
          nodejs-16_x
          (php.withExtensions ({ all, ... }: with all; [
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
          python3
          rpm
          ruby
          rust
          rust-cbindgen
          sqlite
          time
          wasm-bindgen-cli
          (wheel_writer.defaultPackage.${system})
          (windows_sdk.defaultPackage.${system})
          zig
        ];

        CARGO_UNSTABLE_MULTITARGET = "true";

        # aarch64-linux-gnu
        CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = pkgs.writeShellScriptBin "linker" ''
          for arg do
            shift
            [ "$arg" = "-lgcc_s" ] && set -- "$@" "-lunwind" && continue
            set -- "$@" "$arg"
          done
          ZIG_GLOBAL_CACHE_DIR=$(mktemp -d) zig cc -target aarch64-linux-gnu.2.28 $@
        '' + /bin/linker;
        CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS = "-C target-feature=-outline-atomics";
        CC_aarch64_unknown_linux_gnu = pkgs.writeShellScriptBin "cc" ''
          ZIG_GLOBAL_CACHE_DIR=$(mktemp -d) zig cc -target aarch64-linux-gnu.2.28 $@
        '' + /bin/cc;

        # aarch64-linux-musl
        CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER = pkgs.writeShellScriptBin "linker" ''
          for arg do
            shift
            [ "$arg" = "-lgcc_s" ] && set -- "$@" "-lunwind" && continue
            set -- "$@" "$arg"
          done
          ZIG_GLOBAL_CACHE_DIR=$(mktemp -d) zig cc -target aarch64-linux-musl -dynamic $@
        '' + /bin/linker;
        CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUSTFLAGS = "-C target-feature=-crt-static";
        CC_aarch64_unknown_linux_musl = pkgs.writeShellScriptBin "cc" ''
          ZIG_GLOBAL_CACHE_DIR=$(mktemp -d) zig cc -target aarch64-linux-musl $@
        '' + /bin/cc;

        # aarch64-macos
        CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER = pkgs.writeShellScriptBin "linker" ''
          ZIG_GLOBAL_CACHE_DIR=$(mktemp -d) zig cc -target aarch64-macos $@
        '' + /bin/linker;
        CC_aarch64_apple_darwin = pkgs.writeShellScriptBin "cc" ''
          ZIG_GLOBAL_CACHE_DIR=$(mktemp -d) zig cc -target aarch64-macos $@
        '' + /bin/cc;

        # wasm32
        CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER = "lld";

        # x86_64-linux-gnu
        CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER = pkgs.writeShellScriptBin "linker" ''
          for arg do
            shift
            [ "$arg" = "-lgcc_s" ] && set -- "$@" "-lunwind" && continue
            set -- "$@" "$arg"
          done
          ZIG_GLOBAL_CACHE_DIR=$(mktemp -d) zig cc -target x86_64-linux-gnu.2.28 --ld-path=$(which mold) $@
        '' + /bin/linker;
        CC_x86_64_unknown_linux_gnu = pkgs.writeShellScriptBin "cc" ''
          ZIG_GLOBAL_CACHE_DIR=$(mktemp -d) zig cc -target x86_64-linux-gnu.2.28 $@
        '' + /bin/cc;

        # x86_64-linux-musl
        CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER = pkgs.writeShellScriptBin "linker" ''
          for arg do
            shift
            [ "$arg" = "-lgcc_s" ] && set -- "$@" "-lunwind" && continue
            set -- "$@" "$arg"
          done
          ZIG_GLOBAL_CACHE_DIR=$(mktemp -d) zig cc -target x86_64-linux-musl -dynamic $@
        '' + /bin/linker;
        CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUSTFLAGS = "-C target-feature=-crt-static";
        CC_x86_64_unknown_linux_musl = pkgs.writeShellScriptBin "cc" ''
          ZIG_GLOBAL_CACHE_DIR=$(mktemp -d) zig cc -target x86_64-linux-musl $@
        '' + /bin/cc;

        # x86_64-macos
        CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER = pkgs.writeShellScriptBin "linker" ''
          ZIG_GLOBAL_CACHE_DIR=$(mktemp -d) zig cc -target x86_64-macos $@
        '' + /bin/linker;
        CC_x86_64_apple_darwin = pkgs.writeShellScriptBin "cc" ''
          ZIG_GLOBAL_CACHE_DIR=$(mktemp -d) zig cc -target x86_64-macos $@
        '' + /bin/cc;

        # x86_64-windows-gnu
        CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER = pkgs.writeShellScriptBin "linker" ''
          for arg do
            shift
            [ "$arg" = "-lgcc" ] && continue
            [ "$arg" = "-lgcc_eh" ] && continue
            [ "$arg" = "-l:libpthread.a" ] && continue
            set -- "$@" "$arg"
          done
          ZIG_GLOBAL_CACHE_DIR=$(mktemp -d) zig cc -target x86_64-windows-gnu -lstdc++ $@
        '' + /bin/linker;
        CC_x86_64_pc_windows_gnu = pkgs.writeShellScriptBin "cc" ''
          ZIG_GLOBAL_CACHE_DIR=$(mktemp -d) zig cc -target x86_64-windows-gnu $@
        '' + /bin/cc;

        # x86_64-windows-msvc
        AR_x86_64_pc_windows_msvc = "llvm-lib";
        CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER = pkgs.writeShellScriptBin "linker" ''
          lld-link /
            /libpath:$WINDOWS_SDK/clang/lib/x64 \
            /libpath:$WINDOWS_SDK/crt/lib/x64 \
            /libpath:$WINDOWS_SDK/sdk/lib/x64 \
            /libpath:$WINDOWS_SDK/sdk/lib/x64/ucrt \
            /libpath:$WINDOWS_SDK/sdk/lib/x64/um \
            $@
        '' + /bin/linker;
        CC_x86_64_pc_windows_msvc = pkgs.writeShellScriptBin "cc" ''
          clang-cl \
            /I $WINDOWS_SDK/clang/include \
            /I $WINDOWS_SDK/crt/include \
            /I $WINDOWS_SDK/sdk/include \
            /I $WINDOWS_SDK/sdk/include/shared \
            /I $WINDOWS_SDK/sdk/include/ucrt \
            /I $WINDOWS_SDK/sdk/include/um \
            $@
        '' + /bin/cc;
        WINDOWS_SDK = windows_sdk.defaultPackage.${system};
      };
    }
  );
}
