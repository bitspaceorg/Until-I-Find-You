#!/usr/bin/env python3
"""Build docs/data.json — an index of every docs page + its frontmatter.

Consumers (the docs site, search, agents) read data.json and don't have to
parse MDX themselves.
"""

from __future__ import annotations

import json
import pathlib
import sys

try:
    import frontmatter  # type: ignore[import-not-found]
except ImportError:
    sys.stderr.write("missing python-frontmatter; run inside `nix develop`\n")
    sys.exit(2)


ROOT = pathlib.Path(__file__).resolve().parents[2]
DOCS = ROOT / "docs"
OUT = DOCS / "data.json"


def main() -> int:
    entries: list[dict] = []
    for mdx in sorted(DOCS.rglob("*.mdx")):
        post = frontmatter.load(mdx)
        meta = dict(post.metadata)
        meta["path"] = str(mdx.relative_to(ROOT))
        entries.append(meta)

    OUT.write_text(json.dumps({"pages": entries}, indent=2, default=str) + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
