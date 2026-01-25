{
  description = "Rust dev env";
  
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };

				libPath = with pkgs; lib.makeLibraryPath [
					libGL
					libxkbcommon
					wayland
				];
      in {
        devShells.default = pkgs.mkShell {
	  			packages = with pkgs; [
	    			rustc
	    			cargo
	    			rust-analyzer
	  			];

					RUST_LOG = "debug";
					RUST_SRC_PATH =
          	"${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        	LD_LIBRARY_PATH = libPath;
				};
			}
		);
}

