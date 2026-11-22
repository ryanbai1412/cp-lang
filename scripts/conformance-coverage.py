#!/usr/bin/env python3
"""Write spec/conformance/COVERAGE.md: rules, diagnostics and std items vs cases.

A case is positive when it expects `ok`, negative when it expects a diagnostic
or a trap. A rule whose text forbids or traps something needs a negative case;
one cited only by positive cases is listed as happy-path only.
"""
import json
import re
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SPEC = ROOT / "spec"
CONF = SPEC / "conformance"
RULE_DEF = re.compile(r"\(R(\d\d[a-z]?(?:\.\d+)+)\)")
CODE = re.compile(r"\b([ELRW]\d{4})\b")
UNTESTABLE = {
    "E9998": "internal: a construct the compiler cannot lower",
    "E9999": "internal compiler error",
}
NEGATIVE_WORDS = re.compile(
    r"\b(error|errors|rejects?|rejected|cannot|can't|never|not|no|traps?|"
    r"forbidden|illegal|must|only|lint|warns?)\b|`[ELRW]\d{4}`|\b[ELRW]\d{4}\b",
    re.I,
)


def rule_texts():
    rules = {}
    for path in sorted((SPEC / "core").glob("*.md")):
        lines = path.read_text().split("\n")
        start = 0
        for i, line in enumerate(lines):
            stripped = line.strip()
            if not stripped or stripped.startswith(("- ", "#", "|", "```")) or re.match(r"\d+\. ", stripped):
                start = i
            for m in RULE_DEF.finditer(line):
                rid = m.group(1)
                if rid in rules:
                    continue
                chunk = " ".join(l.strip() for l in lines[start : i + 1])
                chunk = chunk[: chunk.find(m.group(0))]
                prev = list(RULE_DEF.finditer(chunk))
                if prev:
                    chunk = chunk[prev[-1].end() :]
                chunk = re.sub(r"^[-#\s]+|^\d+\.\s*", "", chunk).strip()
                rules[rid] = (path.name, i + 1, chunk)
    return rules


def cases():
    out = []
    for path in sorted(CONF.rglob("*.cp")):
        text = path.read_text()
        lines = text.split("\n")
        expect = lines[0].removeprefix("//").strip().removeprefix("expect:").strip()
        cited = []
        for line in lines[1:4]:
            s = line.strip()
            if s.startswith("//") and s[2:].strip().startswith("rule:"):
                cited += s[2:].strip()[5:].split()
        m = CODE.match(expect)
        out.append(
            {
                "id": str(path.relative_to(CONF)),
                "chapter": path.parent.name,
                "expect": expect,
                "code": m.group(1) if m else None,
                "rules": cited,
                "text": text,
            }
        )
    return out


def diag_codes():
    codes = {}
    reg = (ROOT / "crates/cpl-diag/src/registry.rs").read_text()
    for m in re.finditer(r'code: Code\("(\w+)"\),\s*title: "([^"]*)"', reg):
        codes[m.group(1)] = m.group(2)
    for m in re.finditer(r'code: Code\("(\w+)"\),(?:(?!code: Code).)*?mode: "reserved"', reg, re.S):
        UNTESTABLE.setdefault(m.group(1), "reserved in the registry")
    lint = (ROOT / "crates/cpl-lint/src/lib.rs").read_text()
    for m in re.finditer(r'code: Code\("(L\d{4})"\),(?:\s*\w+: [^\n]*\n)*?\s*title: "([^"]*)"', lint):
        codes.setdefault(m.group(1), m.group(2))
    for m in re.finditer(r'"(L\d{4})"', lint):
        codes.setdefault(m.group(1), "")
    catalogue = (SPEC / "diagnostics.md").read_text()
    for line in catalogue.split("\n"):
        m = re.match(r"\| ([ELRW]\d{4})(?:-([ELRW]\d{4}))? \| ([^|]*)\|", line)
        if not m:
            continue
        name = m.group(3).strip()
        if "*(retired)*" in name or line.split("|")[3].strip().startswith("retired"):
            UNTESTABLE.setdefault(m.group(1), "retired")
        if m.group(2):
            lo, hi = int(m.group(1)[1:]), int(m.group(2)[1:])
            for n in range(lo, hi + 1):
                codes.setdefault(f"{m.group(1)[0]}{n:04d}", name)
        else:
            codes[m.group(1)] = codes.get(m.group(1)) or name
    return codes


def std_items():
    items = defaultdict(list)
    for path in sorted((SPEC / "std").glob("*.cp")):
        owner = None
        for line in path.read_text().split("\n"):
            m = re.match(r"(?:pub )?(?:struct|enum|trait|impl)\s+(?:\w+ for )?(\w+)", line)
            if m and not line.startswith(" "):
                owner = m.group(1)
            m = re.match(r"\s*(?:@\w+(?:\([^)]*\))?\s+)*pub (fn|struct|enum|trait|domain|let|const|type)\s+(\w+)", line)
            if not m:
                continue
            kind, name = m.groups()
            indented = line.startswith(" ")
            label = f"{owner}.{name}" if indented and owner else name
            items[path.stem].append((label, name, kind))
    return items


def mentions(label, name, corpus):
    if "." not in label:
        return re.search(rf"(?<![\w]){re.escape(name)}(?![\w])", corpus) is not None
    owner = label.split(".")[0]
    if name == "new":
        return re.search(rf"(?<![\w]){re.escape(owner)}(?:<[^>]*>)?(?:\.new)?\(", corpus) is not None
    return re.search(rf"\.{re.escape(name)}(?![\w])", corpus) is not None


def spec_blocks(all_cases):
    def norm(line):
        if line.lstrip().startswith("///"):
            return ""
        return " ".join(re.sub(r"\s*//.*$", "", line).split())

    sets = [(c["id"], {norm(l) for l in c["text"].split("\n")}) for c in all_cases]
    out = []
    for path in sorted((SPEC / "core").glob("*.md")):
        lines = path.read_text().split("\n")
        i = 0
        while i < len(lines):
            if lines[i].startswith("```"):
                j = i + 1
                while j < len(lines) and not lines[j].startswith("```"):
                    j += 1
                body = [b for b in (norm(x) for x in lines[i + 1 : j]) if b]
                m = re.match(r"<!-- spec-only: (.+) -->$", lines[i - 1].strip()) if i else None
                hit = next((cid for cid, s in sets if body and all(b in s for b in body)), None)
                out.append((path.name, i + 1, hit, m.group(1) if m else None, body[0] if body else ""))
                i = j
            i += 1
    return out


def main():
    rules = rule_texts()
    all_cases = cases()
    scoreboard = json.loads((ROOT / "research/scoreboard.json").read_text())
    status = scoreboard["suites"]["conformance"]["cases"]
    pos, neg = defaultdict(list), defaultdict(list)
    for c in all_cases:
        for r in c["rules"]:
            (neg if c["code"] else pos)[r].append(c["id"])

    by_chapter = defaultdict(list)
    for rid in rules:
        by_chapter[rid.split(".")[0]].append(rid)
    home = {ch: max({rules[r][0] for r in rids}, key=lambda f: sum(rules[r][0] == f for r in rids))
            for ch, rids in by_chapter.items()}

    out = []
    w = out.append
    w("# Conformance coverage")
    w("")
    w("Generated by `scripts/conformance-coverage.py`; do not edit by hand.")
    w("A positive case expects `ok`; a negative case expects a diagnostic or")
    w("a trap. Status comes from `research/scoreboard.json` (`new` = not")
    w("scored yet). A rule is *happy-path only* when its text forbids, traps")
    w("or limits something and no negative case cites it.")
    w("")
    total_cases = len(all_cases)
    passing = sum(1 for c in all_cases if status.get(c["id"]) == "pass")
    uncited = [c["id"] for c in all_cases if not c["rules"] and c["chapter"].startswith("core")]
    w(f"Cases: {total_cases} ({passing} passing). Rules: {len(rules)}.")
    w("")
    w("## Summary per chapter")
    w("")
    for ch in sorted(by_chapter):
        rids = by_chapter[ch]
        chapter_cases = {c for r in rids for c in pos[r] + neg[r]}
        none_ = [r for r in rids if not pos[r] and not neg[r]]
        happy = [r for r in rids if pos[r] and not neg[r] and NEGATIVE_WORDS.search(rules[r][2])]
        nopos = [r for r in rids if neg[r] and not pos[r]]
        w(
            f"- R{ch} ({home[ch]}): {len(rids)} rules, {len(chapter_cases)} cases; "
            f"{len(none_)} uncovered, {len(happy)} happy-path only, {len(nopos)} negative only"
        )
    w("")
    w("## Rules")
    w("")
    w("Each rule: positive / negative case counts, then the gap if any.")
    for ch in sorted(by_chapter):
        w("")
        w(f"### R{ch}: {home[ch]}")
        w("")
        for rid in by_chapter[ch]:
            text = rules[rid][2]
            short = (text[:90] + "...") if len(text) > 90 else text
            p, n = len(pos[rid]), len(neg[rid])
            gap = ""
            if not p and not n:
                gap = " **uncovered**"
            elif not n and NEGATIVE_WORDS.search(text):
                gap = " *happy-path only*"
            elif not p:
                gap = " *negative only*"
            w(f"- R{rid} [{p}+ {n}-]{gap}: {short}")
    w("")
    w("## Diagnostics")
    w("")
    w("Codes from `crates/cpl-diag/src/registry.rs`, `crates/cpl-lint/src/lib.rs`")
    w("and `spec/diagnostics.md`, with the cases that expect each.")
    w("")
    codes = diag_codes()
    by_code = defaultdict(list)
    for c in all_cases:
        if c["code"]:
            by_code[c["code"]].append(c["id"])
    missing_codes = []
    for code in sorted(codes):
        hits = by_code.get(code, [])
        if code in UNTESTABLE:
            mark = f"exempt ({UNTESTABLE[code]})"
        elif not hits:
            missing_codes.append(code)
            mark = "**no case**"
        else:
            mark = f"{len(hits)} case(s)"
        w(f"- {code} {codes[code]}: {mark}")
    w("")
    w(f"{len(missing_codes)} of {len(codes) - len(UNTESTABLE)} testable codes have no negative case.")
    w("")
    w("## Spec code blocks")
    w("")
    w("Every fenced block in `spec/core`. A block is covered when one case")
    w("holds all its code lines (comments aside), and spec-only when the")
    w("line above its fence is `<!-- spec-only: reason -->`.")
    w("")
    blocks = spec_blocks(all_cases)
    gaps = 0
    for name in sorted({b[0] for b in blocks}):
        mine = [b for b in blocks if b[0] == name]
        covered = sum(1 for b in mine if b[2])
        only = sum(1 for b in mine if not b[2] and b[3])
        missing = [b for b in mine if not b[2] and not b[3]]
        gaps += len(missing)
        w(f"- {name}: {len(mine)} blocks, {covered} with a case, {only} spec-only, {len(missing)} neither")
        for b in missing:
            w(f"  - line {b[1]}: `{b[4][:60]}`")
    w("")
    w(f"{gaps} of {len(blocks)} blocks have neither a case nor a spec-only reason.")
    w("")
    w("## Standard library")
    w("")
    w("Every `pub` item in `spec/std`, counted as covered when a conformance")
    w("case names it. `/// example:` lines are also run by the `docs` suite of")
    w("`cpl-conform`; this list is about whole-program cases (edges, traps).")
    w("")
    corpus = "\n".join(c["text"] for c in all_cases)
    total_items = covered_items = 0
    for mod, entries in sorted(std_items().items()):
        seen = {}
        for label, name, kind in entries:
            seen.setdefault(label, (name, kind))
        cov = [l for l, (n, _) in seen.items() if mentions(l, n, corpus)]
        unc = [l for l in seen if l not in cov]
        total_items += len(seen)
        covered_items += len(cov)
        w(f"- **{mod}** ({len(cov)}/{len(seen)} named by a case)")
        if unc:
            w(f"  - uncovered: {', '.join('`' + u + '`' for u in unc)}")
    w("")
    w(f"{covered_items} of {total_items} std items are named by a case.")
    if uncited:
        w("")
        w("## Core cases without a `// rule:` line")
        w("")
        for u in uncited:
            w(f"- {u}")
    text = "\n".join(out) + "\n"
    target = CONF / "COVERAGE.md"
    if "--check" in sys.argv:
        if target.read_text() != text:
            print("spec/conformance/COVERAGE.md is stale; run scripts/conformance-coverage.py")
            return 1
        return 0
    target.write_text(text)
    print(f"wrote {target.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
