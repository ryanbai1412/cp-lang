#!/usr/bin/env python3
"""Write spec/conformance/COVERAGE.md: rules, diagnostics and std items vs cases.

A case is positive when it expects `ok`, negative when it expects a diagnostic
or a trap; a tool case with a `// lacks:` line is also negative. A rule whose text forbids or traps something needs a negative case;
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


def rule_key(rid):
    return [(int(p), "") if p.isdigit() else (int(p[:2]), p[2:]) for p in rid.split(".")]


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
                "lacks": any(ln.startswith("// lacks:") for ln in lines),
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


def fnv_id(text):
    h = 0xCBF29CE484222325
    for b in text.encode():
        h = ((h ^ b) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return f"{h:012x}"[:10]


def marker_above(lines, i):
    """The `<!-- kind: text -->` comment right above line i, as (kind, text)."""
    if i == 0:
        return None, None
    end = i - 1
    if not lines[end].strip().endswith("-->"):
        return None, None
    start = end
    while start >= 0 and "<!--" not in lines[start]:
        start -= 1
    if start < 0:
        return None, None
    body = " ".join(l.strip() for l in lines[start : end + 1])
    m = re.match(r"<!--\s*(spec-only|given):\s*(.*?)\s*-->$", body)
    return (m.group(1), m.group(2)) if m else (None, None)


def spec_blocks(all_cases, block_status=None):
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
                kind, reason = marker_above(lines, i)
                hit = next((cid for cid, s in sets if body and all(b in s for b in body)), None)
                code = "\n".join(lines[i + 1 : j]) + "\n"
                bid = f"core/{path.name}#{fnv_id(code)}"
                if not hit and (block_status or {}).get(bid) == "pass":
                    hit = f"spec-blocks {bid}"
                out.append((path.name, i + 1, hit, reason if kind == "spec-only" else None, body[0] if body else ""))
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
            if c["lacks"]:
                neg[r].append(c["id"])

    by_chapter = defaultdict(list)
    for rid in rules:
        by_chapter[rid.split(".")[0]].append(rid)
    home = {ch: max({rules[r][0] for r in rids}, key=lambda f: sum(rules[r][0] == f for r in rids))
            for ch, rids in by_chapter.items()}

    def forbids(rid):
        return bool(NEGATIVE_WORDS.search(rules[rid][2]))

    def gap(rid):
        if not pos[rid]:
            return "no accept case"
        if forbids(rid) and not neg[rid]:
            return "no reject case"
        return None

    gaps = {rid: g for rid in rules if (g := gap(rid))}

    out = []
    w = out.append
    w("# Conformance coverage")
    w("")
    w("Generated by `scripts/conformance-coverage.py`; do not edit by hand.")
    w("A positive case expects `ok`; a negative case expects a diagnostic or")
    w("a trap, or is a tool case whose `// lacks:` line names output that")
    w("must not appear. Status comes from `research/scoreboard.json` (`new` = not")
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
    w("## Accept and reject coverage per chapter")
    w("")
    w("Accept: rules with a positive case. Reject: rules whose text forbids,")
    w("traps or limits something (the reject column's denominator) with a")
    w("negative case. Complete: rules with no gap in `rule-gaps.txt`.")
    w("")
    w("| Chapter | Rules | Accept | Forbidding | Reject | Complete |")
    w("|---|---|---|---|---|---|")
    tot = [0, 0, 0, 0, 0]
    for ch in sorted(by_chapter):
        rids = by_chapter[ch]
        row = [
            len(rids),
            sum(1 for r in rids if pos[r]),
            sum(1 for r in rids if forbids(r)),
            sum(1 for r in rids if forbids(r) and neg[r]),
            sum(1 for r in rids if r not in gaps),
        ]
        tot = [a + b for a, b in zip(tot, row)]
        n, a, f, rj, c = row
        w(f"| R{ch} | {n} | {a} ({100 * a // n}%) | {f} | {rj} ({100 * rj // max(f, 1)}%) | {c} ({100 * c // n}%) |")
    n, a, f, rj, c = tot
    w(f"| all | {n} | {a} ({100 * a // n}%) | {f} | {rj} ({100 * rj // max(f, 1)}%) | {c} ({100 * c // n}%) |")
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
    w("holds all its code lines (comments aside) or the `spec-blocks` suite")
    w("checks it, and spec-only when the line above its fence is")
    w("`<!-- spec-only: reason -->`. A `<!-- given: code -->` comment there")
    w("holds the declarations the block takes from the prose; `spec-blocks`")
    w("prepends it.")
    w("")
    blocks = spec_blocks(all_cases, scoreboard["suites"].get("spec-blocks", {}).get("cases", {}))
    block_gaps = 0
    for name in sorted({b[0] for b in blocks}):
        mine = [b for b in blocks if b[0] == name]
        covered = sum(1 for b in mine if b[2])
        only = sum(1 for b in mine if not b[2] and b[3])
        missing = [b for b in mine if not b[2] and not b[3]]
        block_gaps += len(missing)
        w(f"- {name}: {len(mine)} blocks, {covered} with a case, {only} spec-only, {len(missing)} neither")
        for b in missing:
            w(f"  - line {b[1]}: `{b[4][:60]}`")
    w("")
    w(f"{block_gaps} of {len(blocks)} blocks have neither a case nor a spec-only reason.")
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
    gaps_file = CONF / "rule-gaps.txt"
    baseline = set()
    if gaps_file.exists():
        baseline = {l.split()[0] for l in gaps_file.read_text().split("\n") if l and not l.startswith("#")}
    new = sorted(r for r in gaps if f"R{r}" not in baseline)
    if new and "--allow-new-gaps" not in sys.argv:
        print("rule coverage went down; these rules lost (or never had) a case:")
        for r in new:
            print(f"  R{r}: {gaps[r]}: {rules[r][0]}:{rules[r][1]}")
        print("add the cases, or rerun with --allow-new-gaps for a rule that can't have one yet")
        return 1
    gaps_text = (
        "# Rules still missing an accept case, or a reject case when their text\n"
        "# forbids something. Generated by scripts/conformance-coverage.py; the\n"
        "# gate fails when a rule not listed here has a gap, so this only shrinks.\n"
        + "".join(f"R{r} {gaps[r]}\n" for r in sorted(gaps, key=rule_key))
    )
    if "--check" in sys.argv:
        stale = [f for f, want in ((target, text), (gaps_file, gaps_text)) if not f.exists() or f.read_text() != want]
        if stale:
            print(", ".join(str(f.relative_to(ROOT)) for f in stale) + " stale; run scripts/conformance-coverage.py")
            return 1
        return 0
    target.write_text(text)
    gaps_file.write_text(gaps_text)
    print(f"wrote {target.relative_to(ROOT)} and {gaps_file.relative_to(ROOT)} ({len(gaps)} rule gaps)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
