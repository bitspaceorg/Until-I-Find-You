# UIFY (Until I Find You) — Build System
# Thin wrapper over cargo.  Nix provides the toolchain.
#
# Usage:
#   make              Debug build
#   make release      Release build
#   make test         Run tests (cargo nextest)
#   make lint         clippy -D warnings
#   make bench        criterion benches
#   make clap         Bundle the CLAP plugin
#   make au           AU via clap-wrappers (macOS)
#   make vst3         VST3 via clap-wrappers
#   make ffi          cdylib + generated C header
#   make wasm         WASM target for uify-ffi
#   make docs-check   Validate docs frontmatter + rebuild index
#   make header       Regenerate include/uify.h
#   make clean        cargo clean + remove result*

CARGO          ?= cargo
NEXTEST        ?= $(CARGO) nextest
WORKSPACE_ARGS := --workspace

# ---------------------------------------------------------------------------
.PHONY: all
all: build

.PHONY: build
build:
	$(CARGO) build $(WORKSPACE_ARGS)

.PHONY: release
release:
	$(CARGO) build $(WORKSPACE_ARGS) --release

.PHONY: test
test:
	$(NEXTEST) run $(WORKSPACE_ARGS)

.PHONY: lint
lint:
	$(CARGO) clippy $(WORKSPACE_ARGS) --all-targets -- -D warnings

.PHONY: bench
bench:
	$(CARGO) bench $(WORKSPACE_ARGS)

# Plugin bundling. Placeholder targets — wire the real bundling commands
# (nih-plug's xtask, clap-wrappers) when the plugin crate lands.
.PHONY: clap
clap:
	$(CARGO) build -p uify-clap-plugin --release

.PHONY: au
au: clap
	@echo "TODO: invoke clap-wrappers → AU packaging"

.PHONY: vst3
vst3: clap
	@echo "TODO: invoke clap-wrappers → VST3 packaging"

.PHONY: ffi
ffi:
	$(CARGO) build -p uify-ffi --release

.PHONY: header
header: ffi
	@echo "header regenerated at include/uify.h (via uify-ffi build.rs)"

.PHONY: wasm
wasm:
	$(CARGO) build -p uify-ffi --target wasm32-unknown-unknown --release

# ---------------------------------------------------------------------------
.PHONY: docs-check
docs-check:
	python tools/docs/validate.py
	python tools/docs/build_index.py

# ---------------------------------------------------------------------------
.PHONY: clean
clean:
	$(CARGO) clean
	rm -f result result-*
