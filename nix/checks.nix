{ ... }:
{
    perSystem =
        { pkgs, uify-deps, ... }:
        {
            # Workspace unit + property tests.
            checks.unit-test = pkgs.stdenv.mkDerivation {
                name = "uify-unit-test";
                src = ./..;
                nativeBuildInputs = uify-deps.nativeBuildInputs;
                buildInputs = uify-deps.buildInputs;
                buildPhase = ''
                    export CARGO_HOME=$PWD/.cargo-home
                    make test
                '';
                installPhase = "touch $out";
            };

            # Clippy across the full workspace, warnings are errors.
            checks.clippy = pkgs.stdenv.mkDerivation {
                name = "uify-clippy";
                src = ./..;
                nativeBuildInputs = uify-deps.nativeBuildInputs;
                buildInputs = uify-deps.buildInputs;
                buildPhase = ''
                    export CARGO_HOME=$PWD/.cargo-home
                    make lint
                '';
                installPhase = "touch $out";
            };
        };
}
