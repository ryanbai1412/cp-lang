#!/usr/bin/env python3
"""Check that relative Markdown links in live docs point to existing repo paths.

Usage:
    python3 scripts/check-doc-links.py [file ...]
"""
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DEFAULT_DOCS = [
    "AGENTS.md",
    "README.md",
    "REPO.md",
    "handbook/process/porting.md",
    "plan/STATUS.md",
    "spec/conformance/README.md",
]
LINK_RE = re.compile(r"(?<!!)\[[^\]]+\]\(([^)]+)\)")


def check_markdown(rel_path: str, text: str, root: str = ROOT) -> list[str]:
    errors: list[str] = []
    base_dir = os.path.dirname(os.path.join(root, rel_path))
    in_fence = False
    for lineno, line in enumerate(text.splitlines(), 1):
        stripped = line.lstrip()
        if stripped.startswith("```"):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        for m in LINK_RE.finditer(line):
            target = m.group(1).strip()
            if (
                not target
                or target.startswith(("http://", "https://", "mailto:", "#"))
                or "<" in target
                or ">" in target
            ):
                continue
            path_part = target.split("#", 1)[0]
            if not path_part:
                continue
            candidate = os.path.normpath(os.path.join(base_dir, path_part))
            if not os.path.exists(candidate):
                errors.append(f"{rel_path}:{lineno}: broken link `{target}`")
    return errors


def main(argv: list[str] | None = None) -> int:
    docs = argv if argv else DEFAULT_DOCS
    errors: list[str] = []
    for rel in docs:
        full = os.path.join(ROOT, rel)
        if not os.path.isfile(full):
            errors.append(f"{rel}: doc file not found")
            continue
        with open(full, encoding="utf-8") as fh:
            errors.extend(check_markdown(rel, fh.read(), ROOT))
    if errors:
        for err in errors:
            print(err, file=sys.stderr)
        return 1
    print(f"ok ({len(docs)} docs checked, 0 broken links)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
