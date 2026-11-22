#!/usr/bin/env python3
"""Give every normative item in a core chapter a rule id.

A normative item is a table body row or a top-level list item outside code
fences, in a section that states rules. Ids are (R<chapter>.<section>.<n>);
items that already carry one keep it, and new ones continue the numbering.
Sections about open questions, exercises, rationale or summaries are skipped.

  scripts/spec-rules.py spec/core/04-indices.md [...]
"""
import re, sys, pathlib

SKIP = re.compile(r"open|exercise|why|picture|in one paragraph|in six sentences|how to evaluate|what this page settles|six models", re.I)
ID = re.compile(r"\(R(\d\d[a-z]?)\.(\d+)\.(\d+)\)")

def tag(path):
    p = pathlib.Path(path)
    chap = re.match(r"(\d\d[a-z]?)", p.name).group(1)
    lines = p.read_text().split("\n")
    sec, skip, fence, ordinal = 0, True, False, 0
    nxt = {}
    for l in lines:
        for m in ID.finditer(l):
            k = int(m.group(2)); nxt[k] = max(nxt.get(k, 0), int(m.group(3)))
    added = 0
    i = 0
    while i < len(lines):
        l = lines[i]
        if l.startswith("```"):
            fence = not fence; i += 1; continue
        if fence: i += 1; continue
        h = re.match(r"^## (.*)", l)
        if h:
            ordinal += 1
            n = re.match(r"(\d+)[a-z]?\.(\d+)?", h.group(1))
            sec = int(n.group(2) or n.group(1)) if n else ordinal
            skip = bool(SKIP.search(h.group(1)))
            i += 1; continue
        if skip or sec == 0: i += 1; continue
        if l.startswith("|") and not re.match(r"^\|[\s:|-]+\|\s*$", l):
            prev = lines[i - 1] if i else ""
            nxtl = lines[i + 1] if i + 1 < len(lines) else ""
            header = re.match(r"^\|[\s:|-]+\|\s*$", nxtl)
            if not header and not ID.search(l):
                nxt[sec] = nxt.get(sec, 0) + 1
                cells = l.rstrip().rstrip("|").rstrip()
                lines[i] = f"{cells} (R{chap}.{sec}.{nxt[sec]}) |"
                added += 1
            i += 1; continue
        if re.match(r"^(- |\d+\. )", l):
            j = i
            while j + 1 < len(lines) and lines[j + 1].startswith("  ") and lines[j + 1].strip():
                j += 1
            block = "\n".join(lines[i:j + 1])
            if not ID.search(block) and not lines[j].rstrip().endswith(":"):
                nxt[sec] = nxt.get(sec, 0) + 1
                lines[j] = lines[j].rstrip() + f" (R{chap}.{sec}.{nxt[sec]})"
                added += 1
            i = j + 1; continue
        i += 1
    p.write_text("\n".join(lines))
    return added

if __name__ == "__main__":
    for f in sys.argv[1:]:
        print(f"{f}: {tag(f)} rule ids added")
