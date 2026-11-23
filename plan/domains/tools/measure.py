#!/usr/bin/env python3
"""Measure the tools domain's metrics (plan/domains/tools/metrics.toml).

  python3 plan/domains/tools/measure.py spec-sync   spec/tools/cli.md vs `cpl --help`
  python3 plan/domains/tools/measure.py help        commands whose `--help` exits 0
  python3 plan/domains/tools/measure.py json        commands whose help offers JSON
  python3 plan/domains/tools/measure.py corpus      cpl check time and cpl build bytes per port
  python3 plan/domains/tools/measure.py golden      end-to-end tests per command (crates/*/tests/*.rs)

Uses $CPL, else target/release/cpl (build it first: cargo build --release -p cpl-cli).
"""
import glob, json, os, re, subprocess, sys, time

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))
CPL = os.environ.get("CPL", os.path.join(ROOT, "target/release/cpl"))


def cpl(*args):
    return subprocess.run([CPL, *args], capture_output=True, text=True, cwd=ROOT)


def commands():
    text = cpl("--help").stdout.split("commands:")[1].split("options:")[0]
    return re.findall(r"^  ([a-z]+)\b", text, re.M)


def spec_sync():
    spec = set(re.findall(r"^\| `cpl (\w+)", open(os.path.join(ROOT, "spec/tools/cli.md")).read(), re.M))
    cli = set(commands())
    print(f"mismatches {len(spec ^ cli)}  spec-only {sorted(spec - cli)}  cli-only {sorted(cli - spec)}")


def help_():
    cs = commands()
    ok = [c for c in cs if cpl(c, "--help").returncode == 0]
    print(f"help {len(ok)}/{len(cs)}  missing {sorted(set(cs) - set(ok))}")


def json_():
    listed = json.loads(cpl("--help", "--json").stdout)["commands"]
    cs = [c["name"] for c in listed]
    ok = [c["name"] for c in listed if c["json"]]
    print(f"json {len(ok)}/{len(cs)}  without {sorted(set(cs) - set(ok))}")


def corpus():
    rows = []
    for f in sorted(glob.glob(os.path.join(ROOT, "corpus/problems/*/port.cp"))):
        t = time.perf_counter()
        ok = cpl("check", f).returncode == 0
        ms = (time.perf_counter() - t) * 1000
        b = subprocess.run([CPL, "build", f, "-o", "-"], capture_output=True, cwd=ROOT)
        rows.append((ms, ok, b.returncode == 0, len(b.stdout), os.path.relpath(f, ROOT)))
    ms = sorted(r[0] for r in rows)
    built = [r for r in rows if r[2]]
    big = max(built, key=lambda r: r[3])
    print(f"ports {len(rows)}  check-clean {sum(r[1] for r in rows)}  built {len(built)}")
    print(f"check wall p50 {ms[len(ms) // 2]:.1f} ms  p95 {ms[int(len(ms) * .95)]:.1f} ms  max {ms[-1]:.1f} ms")
    print(f"C++ max {big[3]} B ({big[4]})  over 32 KB {sum(r[3] > 32768 for r in built)}  over 64 KB {sum(r[3] > 65536 for r in built)}")
    for r in sorted((r for r in built if r[3] > 32768), key=lambda r: -r[3]):
        print(f"  over 32 KB: {r[3]} B {r[4]}")


def golden():
    """Per command, the #[test] fns in crates/*/tests/*.rs that run the cpl
    binary with it: the command's name as the first argument of a call in the
    test, or
    in a helper fn of the same file it calls."""
    cmds = commands()
    arg = re.compile(r'(?:&\[|\.arg\(|\.args\(\[|(?<![.\w])\w+\(&?\[?)\s*(?:&?OsStr::new\()?"([a-z]+)"')
    counts = {c: 0 for c in cmds}
    # What the tests of each command assert: its --help, a usage error,
    # a success (exit 0) and a failing exit code (1 or 3).
    kinds = {
        "help": re.compile(r'helps\(|"--help"'),
        "usage": re.compile(r"misused\(|EXIT_USAGE"),
        "success": re.compile(r"Some\(0\)|\.success\(\)"),
        "exit": re.compile(r"EXIT_ERRORS|EXIT_FAILED|Some\([13]\)|!\w+(?:\.\w+)*\.success\(\)"),
    }
    covered = {c: set() for c in cmds}
    every = []  # tests that walk every command `cpl --help` lists
    for path in sorted(glob.glob(os.path.join(ROOT, "crates/*/tests/*.rs"))):
        text = open(path).read()
        if "CARGO_BIN_EXE_cpl" not in text and "target/release/cpl" not in text:
            continue
        fns = re.split(r"(?<!#\[test\])\n(?=(?:#\[test\]\n)?(?:pub )?fn )", text)
        uses = {}
        for body in fns:
            name = re.match(r"(?:#\[test\]\n)?(?:pub )?fn (\w+)", body)
            if name:
                test = body.startswith("#[test]")
                found = {c for c in arg.findall(body) if c in counts}
                uses[name.group(1)] = (test, body, found)
                if not test:  # the methods of a helper type in the chunk
                    for method in re.findall(r"\n\s+(?:pub )?fn (\w+)", body):
                        uses.setdefault(method, (False, body, found))
        for test, body, direct in uses.values():
            if not test:
                continue
            if '"commands:"' in body and '"--help"' in body:
                every.append(test)
            called = {c for h, (t, _, cs) in uses.items() if not t and re.search(rf"\b{h}\(", body) for c in cs}
            for c in direct | called:
                counts[c] += 1
            helpers = "".join(b for h, (t, b, _) in uses.items() if not t and re.search(rf"\b{h}\(", body))
            for c in direct | called:
                covered[c] |= {k for k, r in kinds.items() if r.search(body + helpers)}
    print("  ".join(f"{c} {n}" for c, n in counts.items()))
    print(f"commands {len(cmds)}  with a test {sum(n > 0 for n in counts.values())}  without {sorted(c for c, n in counts.items() if n == 0)}")
    print(f"tests over every command (help and an unknown option): {len(every)}")
    if every:
        for c in cmds:
            covered[c] |= {"help", "usage"}
    # Commands with no outcome but success or misuse: nothing to exit 1 or 3 on.
    no_failure = {"judges": "only lists profiles; an unknown file or judge is misuse"}
    for c in no_failure:
        covered[c].add("exit")
    full = [c for c in cmds if len(covered[c]) == len(kinds)]
    print(f"commands with help, usage, success and exit tested: {len(full)}/{len(cmds)}")
    for c, why in no_failure.items():
        print(f"  {c}: no failing exit to test ({why})")
    for c in cmds:
        if c not in full:
            print(f"  {c} lacks {', '.join(k for k in kinds if k not in covered[c])}")


if __name__ == "__main__":
    {"spec-sync": spec_sync, "help": help_, "json": json_, "corpus": corpus, "golden": golden}[sys.argv[1]]()
