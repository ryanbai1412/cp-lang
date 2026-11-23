#!/usr/bin/env python3
"""What the dreams need: run `cpl check` over corpus/problems/*/dream.cp and
count the errors by the missing feature or library name they report.

A dream (handbook/process/porting.md) is written in the syntax and library we wish we
had, so its errors are the factory's to-do list. Each error is filed under
a gap:

  syntax: <feature>   E0504 "not implemented yet" (the parser names it)
  library: <name>     a name that is "not in the stdlib (yet)", or a
                      missing field or method
  import: <name>      in the stdlib but not imported
  unnamed: <message>  anything else: the diagnostic does not say what is
                      missing, which is a bug in the diagnostics

Usage: scripts/dream-gaps.py [--json] [--cpl PATH] [DREAM.cp ...]
"""

import argparse
import collections
import glob
import json
import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
HEADER = re.compile(r"^error\[(E\d{4})\]: (.*)$")


def find_cpl(override=None):
    for cand in (override, os.environ.get("CPL")):
        if cand and os.path.isfile(cand):
            return cand
    # The newer build, so a stale release binary doesn't hide a fix.
    built = [os.path.join(ROOT, "target", kind, "cpl") for kind in ("release", "debug")]
    built = [b for b in built if os.path.isfile(b)]
    if built:
        return max(built, key=os.path.getmtime)
    sys.exit("dream-gaps: no cpl binary; run `cargo build --release -p cpl-cli`")


def dreams():
    return sorted(glob.glob(os.path.join(ROOT, "corpus", "problems", "*", "dream.cp")))


def errors(cpl, path):
    """(code, message, help) for each error `cpl check` prints, in order."""
    out = subprocess.run([cpl, "check", path], capture_output=True, text=True).stderr
    found = []
    for line in out.splitlines():
        m = HEADER.match(line)
        if m:
            found.append([m.group(1), m.group(2), ""])
        elif found and line.lstrip().startswith("= help:"):
            found[-1][2] = line.split("= help:", 1)[1].strip()
    return [tuple(e) for e in found]


# A domain or its bounds by name, not a built-in type.
DOM = r"(?!(?:Int|Str|Char|Bool|Float|List|Map|Set|Mod)\b)[A-Z]\w*(?:\.bounds)?"

# Checker rules a dream breaks because it assumes the rule away; each names the rule.
RULES = [
    ("a position of a domain and an Int are different types",
     rf"^expected `Int`, found `\(?{DOM}[`,]|^expected `{DOM}`, found `Int`$|"
     rf"^can't compare a `{DOM}` with a `Int`$|^`[-+*/]` can't be applied to `{DOM}` and `Int`$"),
    ("an array over a domain and a List are different types",
     r"is a \[[^\]]+\]\S*, but .* is a List<|^expected `List<.*>`, found `\["),
    ("an index is a position of the array's own domain",
     r"is indexed by \S+, but|but the index is a|: it may step outside|"
     r"positions of different domains|is a Bounds<\S+>, expected an"),
    ("an array is indexed in every dimension at once", r"takes \d+ indices \(.*\), but \d+ (?:was|were) given"),
    ("an argument the fn changes is passed with `mut`", r"so the argument must be passed as `mut "),
    ("`??` next to arithmetic needs parentheses", r"^`\?\?` mixed with"),
    ("a name `cpl run` can't see from here", r"^`cpl run` doesn't know the name"),
    ("a binding in a `let` condition can't change", r"^a binding in a `let` condition can't be `mut`$"),
]


def gap(code, message, help_text):
    """The missing feature or name an error reports, or None if it names none."""
    m = re.match(r"not implemented yet(?:, and (deferred))?: (.*) \((?:see .*|not in the design)\)$", message)
    if code == "E0504" and m:
        return ("deferred: " if m.group(1) else "syntax: ") + m.group(2)
    m = re.match(r"no (?:`(\S+)` in scope|type or domain named `(\S+)`), and not in the stdlib \(yet\)", message)
    if m:
        return "library: " + (m.group(1) or m.group(2))
    m = re.match(r"no module `(\S+)`, and not in the stdlib \(yet\)", message)
    if m:
        return "library: " + m.group(1)
    m = re.match(r"`(\S+)` has no `(\S+)`, and not in the stdlib \(yet\)", message)
    if m:
        return "library: %s.%s" % (m.group(1), m.group(2))
    m = re.match(r"`(.+)` has no (?:argument|parameter) named `(\S+)`", message)
    if m:
        return "library: %s(%s:)" % (m.group(1), m.group(2))
    if re.match(r"`\S+` is a field of `\S+`, and this method takes no `self`", help_text):
        return "syntax: a method that uses fields without taking `self`"
    if message == "a top-level `return` takes no value":
        return "rule: a value after a top-level `return` (decision 20.3)"
    m = re.match(r"(.+?) has no field or method `(\S+)`", message)
    if m:
        return "library: %s.%s" % (re.sub(r"<.*", "", m.group(1)), m.group(2))
    m = re.match(r"a `(.+)` has no (?:method|member) `(\S+)` that `cpl run` knows", message)
    if m:
        return "library: %s.%s" % (m.group(1), m.group(2))
    m = re.match(r"`cpl run` and `cpl build` can't handle (.+) yet$", message)
    if m:
        return "backend: " + m.group(1)
    m = re.match(r"no module `(\S+)`$", message)
    if m:
        return "import: module " + m.group(1)
    m = re.match(r"`(.+)` is `(.+)` in cp$", message)
    if m:
        return "spelling: `%s` (write `%s`)" % (m.group(1), m.group(2))
    m = re.match(r"CP Lang (\w+) with `(.+)`$", message)
    if m:
        return "spelling: %s (write `%s`)" % (m.group(1), m.group(2))
    if code == "E0309" and message.endswith("can't leave the fn from there"):
        return "rule: `?` that leaves the fn from a branch, lambda or comprehension"
    for rule, pattern in RULES:
        if re.search(pattern, message):
            return "rule: " + rule
    m = re.match(r"`(\S+)` is in the standard library", help_text)
    if m:
        return "import: " + m.group(1)
    return None


def report(cpl, paths):
    files = []
    for path in paths:
        errs = errors(cpl, path)
        files.append({
            "dream": os.path.relpath(path, ROOT),
            "errors": [{"code": c, "message": msg, "gap": gap(c, msg, h)} for c, msg, h in errs],
        })
    first = collections.Counter()
    blocks = collections.Counter()
    total = collections.Counter()
    named_first = 0
    for f in files:
        errs = f["errors"]
        if not errs:
            continue
        lead = errs[0]["gap"] or "unnamed: " + errs[0]["message"]
        first[lead] += 1
        named_first += errs[0]["gap"] is not None
        for key in {e["gap"] or "unnamed: " + e["message"] for e in errs}:
            blocks[key] += 1
        for e in errs:
            total[e["gap"] or "unnamed: " + e["message"]] += 1
    with_errors = sum(1 for f in files if f["errors"])
    return {
        "dreams": len(files),
        "with_errors": with_errors,
        "first_named": named_first,
        "errors": sum(total.values()),
        "errors_named": sum(n for k, n in total.items() if not k.startswith("unnamed: ")),
        "gaps": [{"gap": k, "dreams": blocks[k], "first": first[k], "errors": total[k]}
                 for k in sorted(blocks, key=lambda k: (-blocks[k], -first[k], k))],
        "files": files,
    }


def summary_lines(r, top=10):
    lines = [
        "dreams: %d, %d with errors; first error names the gap in %d/%d; %d/%d errors named"
        % (r["dreams"], r["with_errors"], r["first_named"], r["with_errors"],
           r["errors_named"], r["errors"]),
        "  %-6s %-6s %-6s  %s" % ("dreams", "first", "errors", "gap"),
    ]
    for g in r["gaps"][:top]:
        lines.append("  %-6d %-6d %-6d  %s" % (g["dreams"], g["first"], g["errors"], g["gap"]))
    if len(r["gaps"]) > top:
        lines.append("  … %d more (scripts/dream-gaps.py)" % (len(r["gaps"]) - top))
    return lines


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("paths", nargs="*")
    ap.add_argument("--json", action="store_true", help="print the full report as JSON")
    ap.add_argument("--cpl")
    ap.add_argument("--top", type=int, default=1000)
    args = ap.parse_args()
    r = report(find_cpl(args.cpl), [os.path.abspath(p) for p in args.paths] or dreams())
    if args.json:
        json.dump(r, sys.stdout, indent=2)
        print()
        return
    print("\n".join(summary_lines(r, args.top)))
    unnamed = [(f["dream"], f["errors"][0]["message"]) for f in r["files"]
               if f["errors"] and f["errors"][0]["gap"] is None]
    if unnamed:
        print("first errors that name no gap:")
        for dream, message in unnamed:
            print("  %s: %s" % (dream, message))


if __name__ == "__main__":
    main()
