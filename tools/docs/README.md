# Docs Tooling

All docs automation lives here.

| Script           | Purpose                                                 | Where it runs                                       |
| ---------------- | ------------------------------------------------------- | --------------------------------------------------- |
| `validate.py`    | Link / slug / frontmatter / path-vs-slug consistency    | `docs-build` pre-commit hook + CI `make docs-check` |
| `build_index.py` | Regenerate `docs/data.json` from the mdx tree           | `docs-build` pre-commit hook (auto-adds to commit)  |
| `utils.py`       | Shared frontmatter + path-to-slug helpers used by above | —                                                   |
| `hook.sh`        | Human-invocable entry point equivalent to the hook      | run by hand; pre-commit.nix inlines the same steps  |

## Running locally

```bash
make docs-check                   # both validators
python3 tools/docs/validate.py    # link + slug + frontmatter alone
bash tools/docs/hook.sh           # validate + rebuild + git-add data.json
```

## What `validate.py` enforces

- Every `[text](slug)` resolves to a declared slug.
- Every `.mdx` has the required frontmatter fields (`title, description, slug, author, date`).
- Every `slug:` matches its file path — `docs/reference/core/viewport.mdx` ⇔ `reference-core-viewport`. Catches drift between filesystem and URL space.
- No duplicate slugs.

## Frontmatter contract

Required keys: `title`, `description`, `slug`, `author`, `date`. Optional: `tags` (list), `source`.

`slug` is the kebab-cased path under `docs/` with `/` replaced by `-` and `.mdx` stripped. `docs/architecture/threading.mdx` → `architecture-threading`.

## What's NOT here (on purpose)

- **Code-block compile check.** MoP has a `compile_blocks.py` that compiles fenced C blocks against `libmop.a`. uify is Rust; the equivalent (extract ` ```rust ` blocks + `cargo check`) hasn't been written yet — add it here when/if we want that guarantee.
- **Spell / grammar / style.** Pick a standard tool (vale, prose-lint) if needed; don't roll our own.
