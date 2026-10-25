{
  description = "esp-rs-nix shell";

  inputs = {
    nixpkgs = {
      type = "indirect";
      id = "nixpkgs";
    };

    esp-rs = {
      url = "git+ssh://git@github.com/leighleighleigh/esp-rs-nix.git?ref=refs/tags/v3.1.0";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs: {
    # Instantiate the default dev shell with `nix develop`
    devShells = builtins.mapAttrs (system: pkgs: {
      default =
        let
          esp-rs = inputs.esp-rs.packages.${system}.default;
        in
        pkgs.mkShell rec {
          name = "esp-hal-ulp";

          packages = [
            esp-rs
          ]
          ++ [
            pkgs.cargo-generate
            pkgs.espflash
            pkgs.pkg-config
            pkgs.rust-analyzer
            pkgs.rustup
            pkgs.stdenv.cc
          ];

          shellHook = ''
            # set the shell logline or whatever it's called
            export PS1="''${debian_chroot:+($debian_chroot)}\[\033[01;39m\]\u@\h\[\033[00m\]:\[\033[01;34m\]\W\[\033[00m\]\$ "
            export PS1="(${name})$PS1"

            # This variable is important - it tells rustup where to find the esp toolchain,
            # without needing to copy it into your local ~/.rustup/ folder.
            export RUSTUP_TOOLCHAIN=${esp-rs}

            # Load shell completions for espflash
            if (which espflash >/dev/null 2>&1); then
            . <(espflash completions $(basename $SHELL))
            fi
          '';
        };
    }) inputs.nixpkgs.legacyPackages;
  };
}
