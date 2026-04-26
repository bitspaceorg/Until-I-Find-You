# Docs Tooling

Project-agnostic docs automation. Drop this directory + a `docs/` tree of `.mdx` files into any repo and it works.

| Script           | Purpose                                                 | Where it runs                                          |
| ---------------- | ------------------------------------------------------- | ------------------------------------------------------ |
| `validate.py`    | Link / slug / frontmatter / MDX-syntax consistency      | pre-commit hook + CI                                   |
| `build_index.py` | Regenerate `docs/data.json` from the mdx tree           | pre-commit hook (auto-adds to commit)                  |
| `utils.py`       | Shared frontmatter + path-to-slug + raw-URL helpers     | —                                                      |
| `hook.sh`        | Human-invocable entry point: validate + build + git-add | runs by hand; pre-commit config inlines the same steps |

## Running locally

```bash
python3 tools/docs/validate.py    # link + slug + frontmatter + mdx-syntax checks
python3 tools/docs/build_index.py # regenerate docs/data.json
bash    tools/docs/hook.sh        # all of the above + git-add docs/data.json
```

## Repo layout assumed

- `docs/**/*.mdx` — the source of truth. `docs/data.json` is generated.
- An `origin` git remote, OR a `DOCS_BASE_URL` env var (see _Raw-URL derivation_ below).

No other repo conventions assumed. Project language and build system are irrelevant to this tooling.

## Frontmatter contract

Required keys per `.mdx`: `title`, `description`, `slug`, `author`, `date`. Optional: `tags` (list of strings), `source`.

`slug` is the kebab-cased path under `docs/` with `/` replaced by `-` and `.mdx` stripped. Examples:

| File                               | Required `slug`           |
| ---------------------------------- | ------------------------- |
| `docs/index.mdx`                   | `index`                   |
| `docs/guide.mdx`                   | `guide`                   |
| `docs/architecture/threading.mdx`  | `architecture-threading`  |
| `docs/reference/core/viewport.mdx` | `reference-core-viewport` |

If `docs/foo.mdx` and `docs/foo/` both exist, files under `docs/foo/` are nested as `children` of `foo` in the generated index.

## Raw-URL derivation

Each entry in `data.json` carries a `url` field — the raw-content URL of the source `.mdx`. It's resolved in this order:

1. **`DOCS_BASE_URL`** env var, if set. Used verbatim (trailing slash trimmed). Each entry's URL becomes `<DOCS_BASE_URL>/<file_path>`.
2. **`git remote get-url origin`**, mapped to the forge's raw-content pattern:
    - `github.com/<path>` → `https://github.com/<path>/raw/<branch>`
    - `gitlab.com/<path>` (and other `gitlab.*` hosts) → `https://<host>/<path>/-/raw/<branch>`
    - `bitbucket.org/<path>` → `https://bitbucket.org/<path>/raw/<branch>`

Branch comes from `CI_COMMIT_REF_NAME` if set, otherwise `git rev-parse --abbrev-ref HEAD`, otherwise `main`.

If the remote points at an unknown forge, `build_index.py` exits with a clear error — set `DOCS_BASE_URL` to override.

## What `validate.py` enforces

- Every `[text](slug)` link resolves to a declared slug.
- Every `.mdx` has the required frontmatter keys.
- Every `slug:` equals its path-derived slug (catches drift between filesystem and URL space).
- No duplicate slugs across the tree.
- No unescaped `<digit` outside code blocks (MDX would otherwise try to parse it as JSX and error).

## What's NOT here (on purpose)

- **Code-block compile check.** Language-specific. If your project compiles the fenced blocks in your docs (e.g. `cargo check` for `rust`, `cc` for `c`), add a `compile_blocks.py` here.
- **Spell / grammar / style.** Adopt a standard tool (vale, prose-lint) if needed; don't roll your own.
