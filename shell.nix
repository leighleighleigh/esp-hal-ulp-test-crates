{ pkgs ? import <nixpkgs> {}}:
let 
    esp-rs-src = builtins.fetchGit {
        url = "https://github.com/leighleighleigh/esp-rs-nix";
        # mainline
        rev = "8baa40f096e7f52a10e8438b0bd55ef5dc280164";
        # openocd but with tweaks
        #rev = "ffe1451dcbda038ff117e7b85ac11608406f795e";
    };

    # This will build esp-rs-src, chosen above
    esp-rs = pkgs.callPackage "${esp-rs-src}/esp-rs/default.nix" {
        pkgs = pkgs;
        version = "1.90.0.0"; # Rust version
        crosstool-version = "15.2.0_20251204"; # Cross-compiler toolchain version (GCC)
        binutils-version = "16.3_20250913"; # Binutils version (GDB)
    };

    # OpenOCD fork
    #esp-openocd = pkgs.callPackage "${esp-rs-src}/esp-rs/esp-openocd.nix" {};
in
pkgs.mkShell rec {
    name = "esp-rs-nix";
  

    nativeBuildInputs = [ pkgs.pkg-config ];
    buildInputs = [
        esp-rs 
        #esp-openocd
        #pkgs.espflash
        #pkgs.rust-analyzer
        #pkgs.rustup 
        pkgs.stdenv.cc 
        pkgs.just 
        pkgs.inotify-tools
        pkgs.picocom
        pkgs.libusb1
        # for libudev
        pkgs.systemdMinimal
        # for making fat32 images
        pkgs.mtools
    ];

    LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";

    shellHook = ''
    # custom bashrc stuff
    export PS1_PREFIX="(esp-rs)"
    . ~/.bashrc

    #export LD_LIBRARY_PATH="''${LD_LIBRARY_PATH}:${LD_LIBRARY_PATH}"
    ## this is important - it tells rustup where to find the esp toolchain,
    ## without needing to copy it into your local ~/.rustup/ folder.
    #export RUSTUP_TOOLCHAIN=${esp-rs}

    # Load shell completions for espflash
    if (which espflash >/dev/null 2>&1); then
    . <(espflash completions $(basename $SHELL))
    fi
    '';
}
