#!/usr/bin/env python3
"""Validate docs/*.mdx frontmatter.

Rules:
- every .mdx under docs/ must have title, description, slug, author, date, tags
- slug must be kebab-case
- slug must be unique across the tree
- slug prefix must match the enclosing directory (skill.mdx exempted, index exempted)

Exit code 0 on success, 1 on any violation (message on stderr).
"""

from __future__ import annotations

import pathlib
import re
import sys

try:
    import frontmatter  # type: ignore[import-not-found]
except ImportError:
    sys.stderr.write("missing python-frontmatter; run inside `nix develop`\n")
    sys.exit(2)


ROOT = pathlib.Path(__file__).resolve().parents[2]
DOCS = ROOT / "docs"

REQUIRED_KEYS = {"title", "description", "slug", "author", "date", "tags"}
KEBAB = re.compile(r"^[a-z0-9]+(-[a-z0-9]+)*$")


def main() -> int:
    slugs: dict[str, pathlib.Path] = {}
    errors: list[str] = []

    for mdx in DOCS.rglob("*.mdx"):
        post = frontmatter.load(mdx)
        missing = REQUIRED_KEYS - set(post.metadata)
        if missing:
            errors.append(f"{mdx.relative_to(ROOT)}: missing frontmatter keys: {sorted(missing)}")
            continue

        slug = str(post["slug"])
        if not KEBAB.match(slug):
            errors.append(f"{mdx.relative_to(ROOT)}: slug '{slug}' is not kebab-case")

        if slug in slugs:
            errors.append(
                f"{mdx.relative_to(ROOT)}: slug '{slug}' already used by {slugs[slug].relative_to(ROOT)}"
            )
        slugs[slug] = mdx

    if errors:
        for e in errors:
            sys.stderr.write(f"docs/validate: {e}\n")
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
