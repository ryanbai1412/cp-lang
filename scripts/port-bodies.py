#!/usr/bin/env python3
"""Port working bodies from the 0.2 `stdlib/` into the v0.3 `spec/std` stubs.

For each `fn ... = todo` in `spec/std/<m>.cp`, a fn with the same container,
name and parameter names in `stdlib/<m>.cp` donates its body. Ported fns
whose bodies fail `cpl check` go back to `todo`, until the module has no
errors beyond the ones it had before.

    python3 scripts/port-bodies.py [modules...] [--dry]
"""
import os, re, subprocess, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CPL = ROOT / "target/release/cpl"
ENV = dict(os.environ, CPL_SPEC="1", CPL_STD=str(ROOT / "spec/std"))
FN = re.compile(r"^(\s*)(?:@\w+(?:\([^)]*\))?\s+)*(?:pub\s+)?fn\s+(\w+)")
CONTAINER = re.compile(r"^(?:pub\s+)?(?:struct|enum|trait|impl)\b(.*)")


def indent(line):
    return len(line) - len(line.lstrip(" "))


def container_name(header):
    m = re.search(r"\bfor\s+([A-Za-z_]\w*)", header)
    if m:
        return m.group(1)
    m = re.search(r"([A-Za-z_]\w*)", header)
    return m.group(1) if m else ""


def split_sig(text):
    """(param names, tail after the parameter list) of a signature."""
    i = text.index("fn ")
    j = text.index("(", i)
    depth = 0
    for k in range(j, len(text)):
        c = text[k]
        if c in "([{<":
            depth += 1 if c != "<" or text[k - 1] != "-" else 0
        elif c in ")]}>":
            if c == ">" and text[k - 1] in "-=":
                continue
            depth -= 1
            if depth == 0 and c == ")":
                params, tail = text[j + 1 : k], text[k + 1 :]
                break
    else:
        return None, ""
    names, depth, cur = [], 0, ""
    for c in params + ",":
        if c in "([{<":
            depth += 1
        elif c in ")]}>":
            depth -= 1
        if c in ",;" and depth == 0:
            m = re.match(r"\s*(?:mut\s+)?(\w+)", cur)
            if m:
                names.append(m.group(1))
            cur = ""
        else:
            cur += c
    return names, tail


def items(lines):
    """Every fn: dict(key, start, sig_end, end, body_kind, body)."""
    out, container = [], ""
    i = 0
    while i < len(lines):
        line = lines[i]
        if indent(line) == 0:
            m = CONTAINER.match(line)
            if m:
                container = container_name(m.group(1))
            elif line.strip() and not line.startswith(("@", "///", "//")):
                container = ""
        m = FN.match(line)
        if not m:
            i += 1
            continue
        ind = len(m.group(1))
        if ind == 0 and not CONTAINER.match(line):
            container_here = ""
        else:
            container_here = container if ind > 0 else ""
        # signature: until a line ending in ':' or containing a top-level ' = '
        j, sig = i, line.rstrip()
        while True:
            names, tail = split_sig(sig) if "(" in sig else (None, "")
            if names is not None and (re.search(r"\s=\s", tail) or tail.rstrip().endswith(":")):
                break
            j += 1
            if j >= len(lines) or j - i > 12:
                names = None
                break
            sig += "\n" + lines[j].rstrip()
        if names is None:
            i += 1
            continue
        eq = re.search(r"\s=\s", tail)
        if eq and not tail.rstrip().endswith(":") or (eq and tail[eq.end():].strip()):
            kind, body = "expr", tail[eq.end():].strip()
            end = j + 1
            # an expression body may continue on deeper-indented lines
            while end < len(lines) and lines[end].strip() and indent(lines[end]) > ind:
                body += "\n" + lines[end].rstrip()
                end += 1
        else:
            kind = "block"
            end = j + 1
            while end < len(lines) and (not lines[end].strip() or indent(lines[end]) > ind):
                end += 1
            while end > j + 1 and not lines[end - 1].strip():
                end -= 1
            body = "\n".join(l.rstrip() for l in lines[j + 1 : end])
        out.append(dict(key=(container_here, m.group(2), tuple(names)), name=m.group(2),
                        start=i, sig_end=j, end=end, ind=ind, kind=kind, body=body,
                        todo=(kind == "expr" and body == "todo")))
        i = end
    return out


def check(path):
    r = subprocess.run([str(CPL), "check", str(path)], env=ENV, capture_output=True, text=True, timeout=120)
    lines = set()
    for m in re.finditer(r"-->\s+(\S+?):(\d+):\d+", r.stdout + r.stderr):
        if Path(m.group(1)).name == path.name:
            lines.add(int(m.group(2)))
    count = len(re.findall(r"^error", r.stdout + r.stderr, re.M))
    return count, lines


def render(spec_lines, donors, chosen):
    """spec text with the chosen stubs given their donor bodies."""
    out, prev = [], 0
    spans = []
    stubs = [it for it in items(spec_lines) if it["todo"]]
    for it in stubs:
        if it["key"] not in chosen:
            continue
        d = donors[it["key"]]
        out.extend(spec_lines[prev : it["sig_end"]])
        last = spec_lines[it["sig_end"]]
        k = last.rindex("todo")
        head = last[:k].rstrip()
        assert head.endswith("=")
        start_line = len(out) + 1
        if d["kind"] == "expr" and "\n" not in d["body"]:
            out.append(head + " " + d["body"])
        elif d["kind"] == "expr":
            first, *rest = d["body"].split("\n")
            shift = it["ind"] - d["ind"]
            out.append(head + " " + first)
            out.extend((" " * shift + l) if shift >= 0 else l[-shift:] for l in rest)
        else:
            out.append(head[:-1].rstrip() + ":")
            shift = it["ind"] - d["ind"]
            for l in d["body"].split("\n"):
                out.append(((" " * shift + l) if shift >= 0 else l[-shift:]) if l.strip() else "")
        spans.append((start_line, len(out), it["key"]))
        prev = it["sig_end"] + 1
    out.extend(spec_lines[prev:])
    return out, spans


def port(mod, dry=False):
    spec = ROOT / "spec/std" / f"{mod}.cp"
    donor = ROOT / "stdlib" / f"{mod}.cp"
    if not donor.exists():
        return 0, 0
    spec_lines = spec.read_text().split("\n")
    donors = {}
    for it in items(donor.read_text().split("\n")):
        if not it["todo"] and it["body"].strip():
            donors.setdefault(it["key"], it)
    chosen = {it["key"] for it in items(spec_lines) if it["todo"] and it["key"] in donors}
    candidates = len(chosen)
    if not chosen:
        return 0, 0
    base_count, base_lines = check(spec)
    original = spec.read_text()
    try:
        for _ in range(40):
            text, spans = render(spec_lines, donors, chosen)
            spec.write_text("\n".join(text))
            count, lines = check(spec)
            if count <= base_count:
                break
            bad = {key for (a, b, key) in spans if any(a <= l <= b for l in lines)}
            if not bad:
                chosen = set()
                continue
            chosen -= bad
        else:
            chosen = set()
        text, _ = render(spec_lines, donors, chosen)
        if dry:
            spec.write_text(original)
        else:
            spec.write_text("\n".join(text))
    except Exception:
        spec.write_text(original)
        raise
    return len(chosen), candidates


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    dry = "--dry" in sys.argv
    mods = args or sorted(p.stem for p in (ROOT / "spec/std").glob("*.cp"))
    total = 0
    for m in mods:
        n, c = port(m, dry)
        total += n
        print(f"{m:12} ported {n} of {c} matching stubs", flush=True)
    print(f"port-bodies: {total} bodies ported")


if __name__ == "__main__":
    main()
