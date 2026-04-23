# Formatting configuration: treefmt orchestrates all formatters.
# Run: nix develop -c treefmt
# Pre-commit runs the same config via config.treefmt.build.wrapper.

{ inputs, ... }:
{
    imports = [ inputs.treefmt.flakeModule ];
    perSystem =
        { config, pkgs, ... }:
        {
            treefmt.config = {
                projectRootFile = "flake.nix";
                flakeCheck = false;
                settings.global.excludes = [
                    "docs/data.json"
                    "flake.lock"
                    "Cargo.lock"
                    "target/**"
                ];
                package = pkgs.treefmt;

                programs = {
                    # Rust
                    rustfmt = {
                        enable = true;
                        includes = [ "**/*.rs" ];
                    };

                    # Nix
                    nixfmt = {
                        enable = true;
                        strict = true;
                        width = 180;
                        indent = 4;
                    };

                    # Markdown, MDX, YAML, TOML-adjacent
                    prettier = {
                        enable = true;
                        includes = [
                            "**/*.md"
                            "**/*.mdx"
                            "**/*.yml"
                            "**/*.yaml"
                            "**/*.json"
                        ];
                        excludes = [
                            "target"
                            "result"
                            "result-*"
                            "docs/data.json"
                        ];
                    };

                    # TOML (Cargo.toml, etc.)
                    taplo.enable = true;

                    # Shell
                    shfmt = {
                        enable = true;
                        indent_size = 4;
                        simplify = true;
                    };
                };

                settings.formatter = {
                    prettier.options = [
                        "--print-width"
                        "100"
                        "--tab-width"
                        "4"
                        "--trailing-comma"
                        "es5"
                        "--end-of-line"
                        "lf"
                    ];
                };
            };

            devShells.treefmt = pkgs.mkShell { buildInputs = [ config.treefmt.build.wrapper ] ++ (builtins.attrValues config.treefmt.build.programs); };
        };
}
