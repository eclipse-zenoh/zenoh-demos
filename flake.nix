{
  description = "Eclipse Zenoh Demos — reproducible builds via crane";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, crane, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        # Stable Rust toolchain. Pin to a specific date for reproducibility.
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # Build environment shared by all demos
        commonEnv = {
          CARGO_BUILD_JOBS = "4";
          RUSTFLAGS = "-C linker=clang -C link-arg=-fuse-ld=mold";
        };

        # Native build tools present in all shells/derivations
        commonNativeBuildInputs = with pkgs; [
          clang
          mold
          pkg-config
        ];

        # Runtime libs present in all derivations
        commonBuildInputs = with pkgs; [
          openssl
        ];

        # OpenCV configured for the features used by zcam and zturtle-rust.
        # nixpkgs opencv enables highgui by default; we just need the gtk3 backend.
        opencvPkg = pkgs.opencv.override {
          enableGtk3 = true;
          enableVtk = false;
        };

        # Env vars required by the opencv crate's clang-runtime feature
        opencvEnv = {
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          OPENCV_LINK_LIBS = "opencv_core,opencv_videoio,opencv_imgcodecs,opencv_imgproc,opencv_highgui";
          OPENCV_LINK_PATHS = "${opencvPkg}/lib";
          OPENCV_INCLUDE_PATHS = "${opencvPkg}/include/opencv4";
        };

        # ── Packages ───────────────────────────────────────────────────────────
        #
        # Only demos that currently compile against their pinned zenoh version
        # are exposed as packages. Demos undergoing API rewrite (phase 3) will
        # be added here once their Cargo.toml is updated.

        # computer-vision/zcam/zcam-rust — zenoh 1.9.0, already current
        zcam = craneLib.buildPackage ({
          src = craneLib.cleanCargoSource ./computer-vision/zcam/zcam-rust;
          pname = "zcam";
          version = "0.1.0";

          nativeBuildInputs = commonNativeBuildInputs;
          buildInputs = commonBuildInputs ++ [ opencvPkg ];
        } // commonEnv // opencvEnv);

      in {
        packages = {
          inherit zcam;
          default = zcam;
        };

        # ── Dev shells ─────────────────────────────────────────────────────────

        devShells = {
          # Base shell: Rust toolchain + cargo tools, no native deps beyond openssl.
          # Use for: zenoh-tetris, zenoh-shamir, ROS2 demos (after upgrade).
          default = craneLib.devShell ({
            packages = with pkgs; [
              cargo-watch
              cargo-expand
              mold
            ];
            inputsFrom = [ ];
          } // commonEnv);

          # Shell for demos that link OpenCV (zcam-rust, zturtle-rust).
          with-opencv = pkgs.mkShell ({
            packages = [ rustToolchain ] ++ commonNativeBuildInputs
              ++ commonBuildInputs ++ [ opencvPkg pkgs.mold ];
          } // commonEnv // opencvEnv);

          # Shell for turtlebot3/zlidar-rust which needs udev for serial port access.
          zlidar = pkgs.mkShell ({
            packages = [ rustToolchain ] ++ commonNativeBuildInputs
              ++ commonBuildInputs ++ [ pkgs.udev pkgs.mold ];
          } // commonEnv);

          # Python shell for computer-vision Python demos.
          python-cv = pkgs.mkShell {
            packages = with pkgs; [
              (python3.withPackages (ps: with ps; [
                opencv4
                numpy
                imutils
              ]))
            ];
          };
        };

        # Allow `nix fmt` to format all Nix files in the repo.
        formatter = pkgs.nixfmt-rfc-style;
      });
}
