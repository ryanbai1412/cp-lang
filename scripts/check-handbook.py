#!/usr/bin/env python3
"""Flag handbook text that only makes sense to its writer.

Handbook documents and RFCs are written for an outsider, so they must not
lean on shared history: "as discussed", commit hashes, session links,
relative dates. Fenced code and inline code spans are not checked.

Usage:
    python3 scripts/check-handbook.py [file ...]
"""
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

PHRASES = [
    r"as (we )?discussed",
    r"as mentioned (earlier|above|before)",
    r"the earlier (fix|change|discussion)",
    r"this session",
    r"(yesterday|last night|this morning|earlier today)",
    r"sessions/[0-9a-f]{8,}",
]
PHRASE_RE = re.compile(r"\b(" + "|".join(PHRASES) + r")", re.IGNORECASE)
HASH_RE = re.compile(r"\b(?=[0-9a-f]*[a-f])(?=[0-9a-f]*[0-9])[0-9a-f]{7,40}\b")
CODE_SPAN_RE = re.compile(r"`[^`]*`")


def check_markdown(rel_path: str, text: str) -> list[str]:
    errors: list[str] = []
    in_fence = False
    for lineno, line in enumerate(text.splitlines(), 1):
        if line.lstrip().startswith("```"):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        prose = CODE_SPAN_RE.sub("", line)
        for m in PHRASE_RE.finditer(prose):
            errors.append(f"{rel_path}:{lineno}: context-dependent phrase `{m.group(0)}`")
        for m in HASH_RE.finditer(prose):
            errors.append(f"{rel_path}:{lineno}: commit hash `{m.group(0)}`; say what changed instead")
    return errors


def handbook_files(root: str = ROOT) -> list[str]:
    out = []
    for dirpath, _, files in os.walk(os.path.join(root, "handbook")):
        for name in files:
            if name.endswith(".md"):
                out.append(os.path.relpath(os.path.join(dirpath, name), root))
    return sorted(out)


def main(argv: list[str] | None = None) -> int:
    docs = argv if argv else handbook_files()
    errors: list[str] = []
    for rel in docs:
        with open(os.path.join(ROOT, rel), encoding="utf-8") as fh:
            errors.extend(check_markdown(rel, fh.read()))
    for err in errors:
        print(err, file=sys.stderr)
    if errors:
        print("handbook docs are read by outsiders; see handbook/process/handbook-and-rfcs.md", file=sys.stderr)
        return 1
    print(f"ok ({len(docs)} handbook docs checked)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
