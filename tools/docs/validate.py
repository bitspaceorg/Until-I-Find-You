import os
import re
from utils import collect_docs


def collect_all_slugs(docs):
    """Recursively collect all slugs from the doc tree."""
    slugs = set()
    for doc in docs:
        slugs.add(doc["slug"])
        if "children" in doc:
            slugs.update(collect_all_slugs(doc["children"]))
    return slugs


def validate_links(docs_dir, valid_slugs):
    """Check that all inter-doc links reference valid slugs."""
    # Match markdown links: [text](target) where target has no protocol, no extension
    link_re = re.compile(r"\[([^\]]*)\]\(([^)]+)\)")
    errors = []

    for root, _dirs, files in os.walk(docs_dir):
        for fname in files:
            if not fname.endswith(".mdx"):
                continue
            fpath = os.path.join(root, fname)
            with open(fpath, "r", encoding="utf-8") as f:
                content = f.read()

            for match in link_re.finditer(content):
                target = match.group(2)
                # Skip external URLs, anchors-only, and non-slug links (files with extensions)
                if (
                    target.startswith("http")
                    or target.startswith("#")
                    or "." in target
                ):
                    continue
                # Strip anchor if present: "some-slug#section" -> "some-slug"
                slug_part = target.split("#")[0]
                if slug_part and slug_part not in valid_slugs:
                    errors.append(
                        f"  {fpath}: [{match.group(1)}]({target}) -> unknown slug '{slug_part}'"
                    )

    return errors


def validate_mdx_syntax(docs_dir):
    """Flag MDX hazards — bare `<` followed by a digit outside code blocks.

    MDX parses `<X` as the start of a JSX element. `<0.2` crashes the
    compiler: "Unexpected character `0` before name". Escape as `\\<` or
    `&lt;`, or wrap the expression in backticks / a fenced code block.
    """
    hazard_re = re.compile(r"(?<!\\)<[0-9]")
    errors = []

    for root, _dirs, files in os.walk(docs_dir):
        for fname in files:
            if not fname.endswith(".mdx"):
                continue
            fpath = os.path.join(root, fname)
            with open(fpath, "r", encoding="utf-8") as f:
                lines = f.readlines()

            in_fence = False
            for lineno, raw in enumerate(lines, start=1):
                if raw.lstrip().startswith("```"):
                    in_fence = not in_fence
                    continue
                if in_fence:
                    continue

                # Blank out inline-code spans so `<0.2` in backticks is ignored.
                stripped = re.sub(r"`[^`\n]*`", lambda m: " " * len(m.group()), raw)

                for m in hazard_re.finditer(stripped):
                    col = m.start() + 1
                    errors.append(
                        f"  {fpath}:{lineno}:{col}: unescaped '<{m.group()[1]}' "
                        f"— MDX will parse as JSX. Use '\\<' or '&lt;', or wrap in backticks."
                    )

    return errors


def main():
    docs, _authors, _tags = collect_docs()
    valid_slugs = collect_all_slugs(docs)

    link_errors = validate_links("docs", valid_slugs)
    syntax_errors = validate_mdx_syntax("docs")

    if link_errors:
        print(f"Found {len(link_errors)} broken slug references:")
        for e in link_errors:
            print(e)
    if syntax_errors:
        print(f"Found {len(syntax_errors)} MDX syntax hazards:")
        for e in syntax_errors:
            print(e)
    if link_errors or syntax_errors:
        raise SystemExit(1)

    print("[=== all mdx files are valid ===]")


if __name__ == "__main__":
    main()
