#!/usr/bin/env python3
"""Run spec/conformance: each case's first line says what cpl must do with it.

    // expect: ok
    // expect: ok, output "3"
    // expect: E0601 at 4          (an error with that code on that line)
    // expect: L0601 at 3          (a lint)
    // rule: 05.4.2 05.4.3          (optional: the rules it tests)

Without --strict, cases the current checker does not meet are reported, not
failed. `--rules` lists the rule ids in spec/core that no case cites.
"""
import glob, os, re, subprocess, sys

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
CPL = os.path.join(ROOT, "target", "debug", "cpl")
ENV = dict(os.environ, CPL_SPEC="1", CPL_STD=os.path.join(ROOT, "spec", "std"))
EXPECT = re.compile(r'^//\s*expect:\s*(ok(?:,\s*output\s+"(.*)")?|([ELRW]\d{4})\s+at\s+(\d+)(?:\s*\((\w+)\))?)\s*$')
RULE_REF = re.compile(r"^//\s*rule:\s*(.+)$", re.M)
RULE_DEF = re.compile(r"\(R(\d\d[a-z]?(?:\.\d+)+)\)")


def run_case(path):
    lines = open(path).read().split("\n")
    head = lines[0]
    offset = next(i for i, l in enumerate(lines) if not re.match(r"^//\s*(expect|rule):", l))
    m = EXPECT.match(head.strip())
    if not m:
        return "bad", f"first line is not `// expect: ...`: {head.strip()}"
    check = subprocess.run([CPL, "check", path], capture_output=True, text=True, env=ENV)
    diag = check.stdout + check.stderr
    if m.group(3) and m.group(3).startswith("R"):
        if check.returncode != 0 or "error[" in diag:
            return "fail", f"wanted a run that traps; got: {diag.strip()[:300]}"
        run = subprocess.run([CPL, "run", path], capture_output=True, text=True, env=ENV, input="")
        line = str(int(m.group(4)) + offset)
        if run.returncode != 0 and m.group(3) in run.stderr and f":{line}" in run.stderr:
            return "pass", ""
        return "fail", f"wanted trap {m.group(3)} at {line}; got exit {run.returncode}: {run.stderr.strip()[:300]}"
    if m.group(3):
        code, line = m.group(3), str(int(m.group(4)) + offset)
        hit = re.search(rf"\[{code}\].*?\n.*?:{line}:", diag, re.S) or re.search(rf"{code}.*:{line}:", diag)
        if code in diag and (hit or f":{line}:" in diag):
            return "pass", ""
        return "fail", f"wanted {code} at {line}; got: {diag.strip()[:300] or 'no diagnostic'}"
    if check.returncode != 0 or "error[" in diag:
        return "fail", f"wanted ok; got: {diag.strip()[:300]}"
    if m.group(2) is not None:
        run = subprocess.run([CPL, "run", path], capture_output=True, text=True, env=ENV, input="")
        want = m.group(2).encode().decode("unicode_escape")
        if run.stdout.rstrip("\n") != want:
            return "fail", f"wanted output {want!r}; got {run.stdout.strip()[:200]!r} {run.stderr.strip()[:200]}"
    return "pass", ""


def rules():
    defined = {}
    for f in sorted(glob.glob(os.path.join(ROOT, "spec", "core", "*.md"))):
        for i, line in enumerate(open(f), 1):
            for r in RULE_DEF.findall(line):
                defined.setdefault(r, f"{os.path.relpath(f, ROOT)}:{i}")
    cited = set()
    for f in glob.glob(os.path.join(ROOT, "spec", "conformance", "**", "*.cp"), recursive=True):
        for refs in RULE_REF.findall(open(f).read()):
            cited.update(refs.split())
    return defined, cited


def main(argv):
    strict = "--strict" in argv
    if "--rules" in argv:
        defined, cited = rules()
        missing = [r for r in defined if r not in cited]
        for r in missing:
            print(f"{defined[r]}: R{r} has no conformance case")
        unknown = sorted(cited - set(defined))
        for r in unknown:
            print(f"a case cites R{r}, which no chapter defines")
        print(f"conformance rules: {len(defined) - len(missing)} of {len(defined)} covered, {len(unknown)} unknown")
        return 1 if strict and (missing or unknown) else 0
    cases = sorted(glob.glob(os.path.join(ROOT, "spec", "conformance", "**", "*.cp"), recursive=True))
    tally = {"pass": 0, "fail": 0, "bad": 0}
    for path in cases:
        status, why = run_case(path)
        tally[status] += 1
        if status != "pass":
            print(f"{os.path.relpath(path, ROOT)}: {status}: {why}")
    print(f"conformance: {tally['pass']} of {len(cases)} met, {tally['fail']} not met, {tally['bad']} malformed")
    return 1 if tally["bad"] or (strict and tally["fail"]) else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
