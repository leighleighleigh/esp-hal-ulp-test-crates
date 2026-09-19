{ pkgs ? import <nixpkgs> {}}:
let 
    esp-rs-src = builtins.fetchGit {
        url = "https://github.com/leighleighleigh/esp-rs-nix";
        rev = "b159fc2e7e70854d2c8ccfd48d07564df59681f6";
    };

    # This will build esp-rs-src, chosen above
    esp-rs = pkgs.callPackage "${esp-rs-src}/package.nix" { };

    # OpenOCD fork
    #esp-openocd = pkgs.callPackage "${esp-rs-src}/esp-rs/esp-openocd.nix" {};
in
pkgs.mkShell rec {
    name = "esp-rs-nix";
  

    nativeBuildInputs = [ pkgs.pkg-config ];
    buildInputs = [
        #esp-openocd
        #pkgs.espflash
        #pkgs.rustup 
        esp-rs 
        pkgs.rust-analyzer
        pkgs.stdenv.cc 
        pkgs.just 
        pkgs.just-lsp
        pkgs.inotify-tools
        pkgs.picocom
        pkgs.libusb1
        # for libudev
        pkgs.systemdMinimal
        # for documents built by rust
        pkgs.ncurses
    ];

    LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";

    shellHook = ''
    # set the shell logline or whatever it's called
    export PS1="''${debian_chroot:+($debian_chroot)}\[\033[01;39m\]\u@\h\[\033[00m\]:\[\033[01;34m\]\W\[\033[00m\]\$ "
    export PS1="(esp-hal-ulp-tests)$PS1"

    # Load shell completions for espflash
    if (which espflash >/dev/null 2>&1); then
    . <(espflash completions $(basename $SHELL))
    fi
    '';
}
