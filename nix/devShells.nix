{ ... }:
{
    perSystem =
        {
            config,
            pkgs,
            self',
            uify-deps,
            ...
        }:
        {
            devShells.default = pkgs.mkShell {
                nativeBuildInputs = uify-deps.nativeBuildInputs;
                buildInputs = uify-deps.buildInputs;
                inputsFrom = [
                    self'.devShells.treefmt
                    self'.devShells.precommit
                ];
                shellHook = ''
                    ${config.pre-commit.installationScript}
                    echo "Until I Find You — development shell"
                    echo "  Build:           make"
                    echo "  Build (release): make release"
                    echo "  Test:            make test"
                    echo "  Bench:           make bench"
                    echo "  Clippy:          make lint"
                    echo "  Format:          treefmt"
                    echo "  Docs check:      make docs-check"
                    echo "  Plugin (CLAP):   make clap"
                '';
            };

            # Minimal CI shell: toolchain + test runner, no treefmt / pre-commit.
            devShells.ci = pkgs.mkShell {
                nativeBuildInputs = uify-deps.nativeBuildInputs;
                buildInputs = uify-deps.buildInputs;
            };
        };
}
