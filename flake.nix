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

        # computer-vision/zcam/zcam-rust — zenoh 1.9.0
        zcam = craneLib.buildPackage ({
          src = craneLib.cleanCargoSource ./computer-vision/zcam/zcam-rust;
          pname = "zcam";
          version = "0.1.0";
          nativeBuildInputs = commonNativeBuildInputs;
          buildInputs = commonBuildInputs ++ [ opencvPkg ];
        } // commonEnv // opencvEnv);

        # turtlebot3/zlidar-rust — 1.4.0 → 1.9.0 (version bump)
        zlidar = craneLib.buildPackage ({
          src = craneLib.cleanCargoSource ./turtlebot3/zlidar-rust;
          pname = "zlidar";
          version = "0.1.0";
          nativeBuildInputs = commonNativeBuildInputs;
          buildInputs = commonBuildInputs ++ [ pkgs.udev ];
        } // commonEnv);

        # ROS2/zenoh-rust-replay — 1.0.3 → 1.9.0 (async-std → tokio)
        zenoh-rust-replay = craneLib.buildPackage ({
          src = craneLib.cleanCargoSource ./ROS2/zenoh-rust-replay;
          pname = "ros2-replay";
          version = "0.0.1";
          nativeBuildInputs = commonNativeBuildInputs;
          buildInputs = commonBuildInputs;
        } // commonEnv);

        # zenoh-dds-interop/shapes_demo — 0.11 → 1.9.0 (drop zenoh-util)
        zenoh-dds-shapes = craneLib.buildPackage ({
          src = craneLib.cleanCargoSource ./zenoh-dds-interop/shapes_demo;
          pname = "shapes-demo";
          version = "0.1.0";
          nativeBuildInputs = commonNativeBuildInputs;
          buildInputs = commonBuildInputs;
        } // commonEnv);

        # zenoh-shamir — 0.10.0-rc → 1.9.0 (full async rewrite)
        zenoh-shamir-put = craneLib.buildPackage ({
          src = craneLib.cleanCargoSource ./zenoh-shamir;
          pname = "zenoh_put_shamir";
          version = "0.0.1";
          cargoExtraArgs = "--bin zenoh_put_shamir";
          nativeBuildInputs = commonNativeBuildInputs;
          buildInputs = commonBuildInputs;
        } // commonEnv);

        zenoh-shamir-queryable = craneLib.buildPackage ({
          src = craneLib.cleanCargoSource ./zenoh-shamir;
          pname = "zenoh_queryable_shamir";
          version = "0.0.1";
          cargoExtraArgs = "--bin zenoh_queryable_shamir";
          nativeBuildInputs = commonNativeBuildInputs;
          buildInputs = commonBuildInputs;
        } // commonEnv);

        # zenoh-tetris — 0.10.0-rc → 1.9.0 (sync .wait() API)
        zenoh-tetris-server = craneLib.buildPackage ({
          src = craneLib.cleanCargoSource ./zenoh-tetris;
          pname = "zenoh-tetris-server";
          version = "0.1.0";
          cargoExtraArgs = "--bin server";
          nativeBuildInputs = commonNativeBuildInputs;
          buildInputs = commonBuildInputs;
        } // commonEnv);

        zenoh-tetris-client = craneLib.buildPackage ({
          src = craneLib.cleanCargoSource ./zenoh-tetris;
          pname = "zenoh-tetris-client";
          version = "0.1.0";
          cargoExtraArgs = "--bin client";
          nativeBuildInputs = commonNativeBuildInputs;
          buildInputs = commonBuildInputs;
        } // commonEnv);

        zenoh-tetris-hotseat = craneLib.buildPackage ({
          src = craneLib.cleanCargoSource ./zenoh-tetris;
          pname = "zenoh-tetris-hotseat";
          version = "0.1.0";
          cargoExtraArgs = "--bin hot_seat";
          nativeBuildInputs = commonNativeBuildInputs;
          buildInputs = commonBuildInputs;
        } // commonEnv);

        # ROS2/zenoh-rust-teleop — 0.10.0-rc → 1.9.0 (async rewrite, clap v4)
        zenoh-rust-teleop = craneLib.buildPackage ({
          src = craneLib.cleanCargoSource ./ROS2/zenoh-rust-teleop;
          pname = "ros2-teleop";
          version = "0.0.1";
          nativeBuildInputs = commonNativeBuildInputs;
          buildInputs = commonBuildInputs;
        } // commonEnv);

        # turtlebot3/zturtle-rust — 0.10.0-rc → 1.9.0 (sync .wait() + opencv 0.94)
        zturtle = craneLib.buildPackage ({
          src = craneLib.cleanCargoSource ./turtlebot3/zturtle-rust;
          pname = "zturtle";
          version = "0.1.0";
          nativeBuildInputs = commonNativeBuildInputs;
          buildInputs = commonBuildInputs ++ [ opencvPkg pkgs.udev ];
        } // commonEnv // opencvEnv);

      in {
        packages = {
          inherit zcam zlidar zenoh-rust-replay zenoh-dds-shapes
            zenoh-shamir-put zenoh-shamir-queryable
            zenoh-tetris-server zenoh-tetris-client zenoh-tetris-hotseat
            zenoh-rust-teleop zturtle;
          default = zcam;
        };

        # ── Dev shells ─────────────────────────────────────────────────────────

        devShells = {
          # Base shell: Rust toolchain + cargo tools, no native deps beyond openssl.
          # openssh included so whippet-cli (which calls ssh) works from inside the sandbox.
          default = craneLib.devShell ({
            packages = with pkgs; [
              cargo-watch
              cargo-expand
              mold
              openssh
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
