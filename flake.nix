{
	description = "A very basic flake";

	inputs = {
		nixpkgs.url     = "github:nixos/nixpkgs/nixpkgs-unstable";
		flake-utils.url = "github:numtide/flake-utils";
	};

	outputs = { self, nixpkgs, flake-utils }:
		flake-utils.lib.eachDefaultSystem(system:
			let
				pkgs = nixpkgs.legacyPackages.${system};
				lib = nixpkgs.lib;
			in rec {
				devShells.default = pkgs.mkShell {
					nativeBuildInputs = with pkgs; [
						gnumake
						cargo
						rustc
						rustfmt
						clippy
						rust-analyzer
						cargo-expand

						mangohud

						pkg-config
						udev
						alsa-lib
						xorg.libXcursor
						xorg.libXrandr
						xorg.libXi
						vulkan-tools
						vulkan-headers
						vulkan-loader
						vulkan-validation-layers
					];
					shellHook = ''
						export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath [
							pkgs.alsaLib
							pkgs.udev
							pkgs.vulkan-loader
						]}"
					'';
				};
			}
		);
}
