# Until I Find You

**UIFY** — modular, latency-first motion-tracking engine in Rust.

UIFY produces time-indexed streams of geometric primitives — points, bounding boxes, 6DoF poses, face landmarks, rotoscope contours — and ships them to other programs over OSC, MIDI/MPE, shared memory, a C ABI, or a first-class CLAP plugin for sample-accurate DAW automation.

## Start here

Read **[docs/skill.mdx](docs/skill.mdx)** — self-contained entry point covering build, the `Tracker<G>` trait, the minimal pipeline, the DAW hand-control example, and the real-time rules. It is the source of truth; this README is a marketing page.

## Built-in trackers

| Tracker        | Geometry                                | Typical use                             |
| -------------- | --------------------------------------- | --------------------------------------- |
| point / ball   | `Vec2` / `Vec3`                         | audio automation, cursor control        |
| bounding box   | axis-aligned or oriented (`SE(2), w, h)` | detection gating, VFX masks            |
| plane          | `SE(3)` (homography + PnP)              | AR, virtual camera body, compositing    |
| face           | landmarks + `SE(3)` head pose + blendshapes | expression capture, retargeting     |
| rotoscope      | contour / signed-distance mask          | matte extraction, VFX                   |

Every tracker implements the same trait and produces `Sample<G, C>` values — value, tangent-space covariance, confidence, host-monotonic timestamp.

## Build

```bash
nix develop                   # development shell (installs pre-commit hooks)
make                          # build the full workspace
make test                     # run unit + property + synthetic GT tests
make lint                     # clippy -D warnings
make clap                     # build the CLAP plugin
make docs-check               # validate docs frontmatter + rebuild index
```

Without Nix:

```bash
rustup toolchain install 1.83
make
```

## Architecture

```
Consumers (DAW, external apps)
        ▲
Transports:  OSC │ MIDI / MPE │ shm │ FFI  │  CLAP plugin
        ▲
Trackers:    point │ bbox │ plane │ face │ roto
        ▲
Core:        Tracker trait, manifolds (SO(3)/SE(3)/SL(3)), filters, pipeline
        ▲
Runtime:     camera │ inference │ lock-free ring buffer
```

Three threads. One lock-free bridge. No allocations on the audio thread. See [docs/architecture.mdx](docs/architecture.mdx) for the layer contract and [docs/architecture/threading.mdx](docs/architecture/threading.mdx) for the thread model.

## Extending

1. Create `crates/uify-<name>/`, depend on `uify-core`.
2. Implement `uify_core::Tracker`.
3. (Optional) Register in `uify-clap-plugin` to expose as DAW parameters.
4. Document at `docs/reference/trackers/<name>.mdx`.

See [docs/architecture/extension.mdx](docs/architecture/extension.mdx) for the full guide.

## Documentation

- **[docs/skill.mdx](docs/skill.mdx)** — source-of-truth entry for building with UIFY
- [docs/index.mdx](docs/index.mdx) — documentation root
- [docs/architecture.mdx](docs/architecture.mdx) — system architecture
- [docs/reference.mdx](docs/reference.mdx) — crate API reference
- [docs/development.mdx](docs/development.mdx) — build, platform, contributor guide

## License

Apache-2.0.
