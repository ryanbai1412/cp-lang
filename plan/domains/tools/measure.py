#!/usr/bin/env python3
"""Measure the tools domain's metrics (plan/domains/tools/metrics.toml).

  python3 plan/domains/tools/measure.py spec-sync   spec/tools/cli.md vs `cpl --help`
  python3 plan/domains/tools/measure.py help        commands whose `--help` exits 0
  python3 plan/domains/tools/measure.py json        commands whose help offers JSON
  python3 plan/domains/tools/measure.py corpus      cpl check time and cpl build bytes per port

Uses $CPL, else target/release/cpl (build it first: cargo build --release -p cpl-cli).
"""
import glob, os, re, subprocess, sys, time

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
    cs = commands()
    ok = [c for c in cs if "json" in cpl(c, "--help").stdout.lower()]
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


if __name__ == "__main__":
    {"spec-sync": spec_sync, "help": help_, "json": json_, "corpus": corpus}[sys.argv[1]]()
