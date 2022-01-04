{
  inputs = {
    nixpkgs = {
      url = "github:nixos/nixpkgs";
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
  outputs =
    inputs: inputs.flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import inputs.nixpkgs {
        inherit system;
        overlays = [
          (self: super: {
            abuild = super.abuild.overrideAttrs (old: {
              patches = [
                (pkgs.fetchpatch {
                  url = "https://gitlab.alpinelinux.org/alpine/abuild/-/merge_requests/130.patch";
                  sha256 = "sha256-9+MpH9HTNDzfRd7vwTD2yU7guIYScAuGMpsqSdvZ9p4=";
                })
              ];
              patchPhase = null;
              postPatch = old.patchPhase;
              propagatedBuildInputs = with self; [
                apk-tools
                fakeroot
                libressl
                pax-utils
              ];
            });
            rpm = super.rpm.overrideAttrs (_: {
              patches = [
                (pkgs.fetchpatch {
                  url = "https://github.com/rpm-software-management/rpm/pull/1775.patch";
                  sha256 = "sha256-WYlxPGcPB5lGQmkyJ/IpGoqVfAKtMxKzlr5flTqn638=";
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
                rev = "a76910b691179390b122b9c42a5575c00d40583a";
                hash = "sha256-f8CT7BBc1KOLJJs/4OoIqK6/m1lkfLcN3w/bn5enqBQ=";
              };
              patches = [
                (self.fetchpatch {
                  url = "https://github.com/ziglang/zig/pull/9771.patch";
                  sha256 = "sha256-AaMNNBET/x0f3a9oxpgBZXnUdKH4bydKMLJfXLBmvZo=";
                })
              ];
            });
          })
        ];
      };
      rust =
        let
          toolchain = {
            channel = "nightly";
            date = "2022-01-01";
            sha256 = "sha256-XUPcV51ws0fwqPrqglxfQ2j1KdxnYkDXCoMaZCbgtuI=";
          };
        in
        with inputs.fenix.packages.${system}; combine (with toolchainOf toolchain; [
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
      windows_sdk = pkgs.runCommand "windows_sdk"
        {
          nativeBuildInputs = with pkgs; [
            (inputs.windows_sdk.defaultPackage.${system})
          ];
          outputHashMode = "recursive";
          outputHash = "sha256-0tMWa9FcZYLbkBiHQCDFvxoY2sf0/A1FxhCqgUVobx4=";
        }
        ''
          windows_sdk \
            --manifest-url https://download.visualstudio.microsoft.com/download/pr/b763973d-da6e-4025-834d-d8bc48e7d37f/81eb5576c4f6514b8744516eac345f5bb062723cec3dbd36aba0594a50482ef3/VisualStudio.vsman \
            --package Microsoft.VisualStudio.VC.Llvm.Clang \
            --package Microsoft.VisualStudio.Component.VC.Tools.x86.x64 \
            --package Microsoft.VisualStudio.Component.Windows10SDK.19041 \
            --cache $(mktemp -d) \
            --output $out
        '';
      macos_sdk = builtins.fetchTarball {
        url = "https://github.com/phracker/MacOSX-SDKs/releases/download/11.3/MacOSX11.3.sdk.tar.xz";
        sha256 = "1ahldqlw9v8wixj9hc3knx3f58178rdkihywvlikss472j2md086";
      };
    in
    {
      devShell = pkgs.mkShell {
        packages = with pkgs; [
          abuild
          cachix
          cargo-insta
          cargo-outdated
          clang_12
          createrepo_c
          doxygen
          dpkg
          elixir
          gnupg
          go
          lld_12
          llvm_12
          mold
          nodejs
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
          zig
        ];

        MACOS_SDK = macos_sdk;
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
          ZIG_GLOBAL_CACHE_DIR=$(mktemp -d) zig cc -target aarch64-macos.11 -L${inputs.nixpkgs.legacyPackages.aarch64-darwin.libiconv}/lib -F${macos_sdk}/System/Library/Frameworks -Wl,-undefined=dynamic_lookup $@
        '' + /bin/linker;
        CC_aarch64_apple_darwin = pkgs.writeShellScriptBin "cc" ''
          ZIG_GLOBAL_CACHE_DIR=$(mktemp -d) zig cc -target aarch64-macos.11 $@
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
          ZIG_GLOBAL_CACHE_DIR=$(mktemp -d) zig cc -target x86_64-macos.11 -L${inputs.nixpkgs.legacyPackages.x86_64-darwin.libiconv}/lib -F${macos_sdk}/System/Library/Frameworks -Wl,-undefined=dynamic_lookup $@
        '' + /bin/linker;
        CC_x86_64_apple_darwin = pkgs.writeShellScriptBin "cc" ''
          ZIG_GLOBAL_CACHE_DIR=$(mktemp -d) zig cc -target x86_64-macos.11 $@
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
          lld-link \
            /libpath:"${windows_sdk}/VC/Tools/Llvm/lib" \
            /libpath:"${windows_sdk}/VC/Tools/MSVC/14.29.30133/lib/x64" \
            /libpath:"${windows_sdk}/Windows Kits/10/Lib/10.0.19041.0/ucrt/x64" \
            /libpath:"${windows_sdk}/Windows Kits/10/Lib/10.0.19041.0/um/x64" \
            $@
        '' + /bin/linker;
        CC_x86_64_pc_windows_msvc = pkgs.writeShellScriptBin "cc" ''
          clang-cl \
            /I "${windows_sdk}/VC/Tools/Llvm/lib/clang/12.0.0/include" \
            /I "${windows_sdk}/VC/Tools/MSVC/14.29.30133/include" \
            /I "${windows_sdk}/Windows Kits/10/Include/10.0.19041.0/ucrt" \
            /I "${windows_sdk}/Windows Kits/10/Include/10.0.19041.0/um" \
            /I "${windows_sdk}/Windows Kits/10/Include/10.0.19041.0/shared" \
            $@
        '' + /bin/cc;
      };
    }
    );
}
