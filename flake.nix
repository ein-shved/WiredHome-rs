{
  description = ''
    Own Smart Home decentralized wired project based on stm32 controllers and
    written on rust
  '';
  inputs = {
    nixpkgs.url = github:NixOS/nixpkgs/nixos-23.05;
    stm32.url = github:ein-shved/nix-stm32;
    stm32.inputs.nixpkgs.follows = "nixpkgs";
  };
  outputs = { self, stm32, nixpkgs }:
    stm32.mkRustFirmware {
      pname = "WiredHome-rs";
      version = "0.1.0";
      mcu = stm32.mcus.stm32f103;
      nightly = true;
      buildPackage = false; # TODO (Shvedov) failed to build embassy libs
      # with nix as dependency to this package
      src = ./embassy/.;
    };
}
