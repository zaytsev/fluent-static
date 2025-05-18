{pkgs, ...}: {
  # Enable devenv language support
  languages.rust = {
    enable = true;
    # Use stable channel
    channel = "stable";
  };

  # Additional packages from the original flake
  packages = with pkgs; [
    # Cargo tools
    cargo-edit
    cargo-update
    cargo-geiger
    cargo-outdated
    cargo-audit
    cargo-expand

    # Other tools
    cocogitto
    pkg-config
    icu74
    clang
  ];

  # Environment variables
  env.LIBCLANG_PATH = "${pkgs.llvmPackages_18.libclang.lib}/lib";
}
