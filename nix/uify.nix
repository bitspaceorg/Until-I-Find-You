{ inputs, ... }:
{
    perSystem =
        { pkgs, system, ... }:
        let
            fenix = inputs.fenix.packages.${system};
            # Read rust-toolchain.toml to pick a single source of truth for the toolchain.
            toolchain = fenix.fromToolchainFile {
                file = ../rust-toolchain.toml;
                sha256 = "sha256-AJ6LX/Q/Er9kS15bn9iflkUwcgYqRQxiOIL2ToVAXaU=";
            };
            python = pkgs.python3.withPackages (ps: [ ps.python-frontmatter ]);
        in
        {
            _module.args.uify-deps = {
                nativeBuildInputs = [
                    toolchain
                ]
                ++ (with pkgs; [
                    pkg-config
                    cmake
                    python
                    cargo-nextest
                    cargo-deny
                    cargo-audit
                ])
                ++ pkgs.lib.optionals (!pkgs.stdenv.isDarwin) [
                    pkgs.valgrind
                    pkgs.gdb
                ];

                buildInputs =
                    with pkgs;
                    [
                        # Vision / ML runtime libraries consumed via -sys crates.
                        onnxruntime
                    ]
                    ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
                        pkgs.v4l-utils
                        pkgs.alsa-lib
                        pkgs.libxkbcommon
                    ];
                # Darwin SDK / frameworks (AVFoundation, CoreML, Metal, AudioToolbox, etc.)
                # come through the modern stdenv automatically — the legacy
                # `darwin.apple_sdk.frameworks.*` attribute path was removed in nixpkgs 26.05.
                # Rust sys-crates declare their own `#[link(framework = "…")]` so nothing
                # extra is needed here; if a specific SDK version is required later,
                # add `pkgs.apple-sdk_15` (or similar) to buildInputs.
            };

            # packages.default is intentionally omitted for now. Cargo workspaces
            # are best built inside the devShell with `make` / `cargo build`.
            # When we need a Nix-reproducible build artifact, wire in `crane`
            # (https://github.com/ipetkov/crane) and add a `packages.uify-ffi`
            # for the C-ABI shared library and a `packages.uify-clap-plugin` for the
            # plugin bundle.
        };
}
