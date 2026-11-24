"""Re-run passing conformance cases after meaning-preserving edits.

Each `// expect: ok, output "..."` case that passes in research/scoreboard.json
is rewritten by every transform below and run again with `cpl run`; the
output must not change. A change is a compiler bug the hand-written case
could not see (equivalence modulo inputs):

  fmt      the source after `cpl fmt --stdin`
  crlf     every line ending written as CRLF (R01: same program on Windows)
  trailing two blank lines and a comment appended
  spaced   a blank line after every line
  comments a comment after every code line, at that line's indentation

Cases that read input, run a tool or use other files are skipped.

Enforces handbook/guidelines/conformance-checklist.md#conformancer6-results-agree-across-engines-with-and-without-sanitizers
"""
from __future__ import annotations

import json
import os
import re
import subprocess
import tempfile
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import specgate
from specgate import Problem

HELP = "every passing ok case keeps its output after fmt, CRLF, blank lines and comments"


def cpl() -> str:
    return os.environ.get("CPL", str(specgate.ROOT / "target/debug/cpl"))

EXPECT = re.compile(r'^// expect: ok, output "((?:[^"\\]|\\.)*)"\s*$')
SKIP = ("// input:", "// tool:", "// file:", "// judge:", "// rewritten:", "// output:")


def header(src: str) -> list[str]:
    out = []
    for line in src.splitlines():
        if not line.startswith("//"):
            break
        out.append(line)
    return out


def expected(src: str) -> str | None:
    head = header(src)
    if not head or any(h.startswith(SKIP) for h in head):
        return None
    m = EXPECT.match(head[0])
    return json.loads('"' + m.group(1) + '"') if m else None


def fmt(src: str) -> str | None:
    r = subprocess.run([cpl(), "fmt", "--stdin"], input=src, capture_output=True,
                       text=True, timeout=60)
    return r.stdout if r.returncode == 0 and r.stdout else None


TRANSFORMS = {
    "fmt": fmt,
    "crlf": lambda s: s.replace("\n", "\r\n"),
    "trailing": lambda s: s + "\n\n// end\n",
    "spaced": lambda s: "".join(line + "\n\n" for line in s.split("\n")),
    "comments": lambda s: "".join(
        line + "\n" + (line[:len(line) - len(line.lstrip())] + "// m\n"
                       if line.strip() and not line.lstrip().startswith("//") else "")
        for line in s.split("\n")),
}


def run_src(src: str) -> tuple[int, str, str]:
    with tempfile.TemporaryDirectory() as d:
        p = Path(d) / "case.cp"
        p.write_bytes(src.encode())
        r = subprocess.run([cpl(), "run", str(p)], stdin=subprocess.DEVNULL,
                           capture_output=True, text=True, timeout=120)
        return r.returncode, r.stdout, r.stderr


def check(name: str) -> list[str]:
    src = (specgate.CONFORMANCE / name).read_text(encoding="utf-8")
    want = expected(src)
    if want is None:
        return []
    bad = []
    for t, f in TRANSFORMS.items():
        new = f(src)
        if new is None:
            bad.append(f"{name}: {t}: cpl fmt failed")
            continue
        code, out, err = run_src(new)
        if code != 0 or out.rstrip("\n") != want.rstrip("\n"):
            got = (err.strip().splitlines() or [out.strip()])[0][:160]
            bad.append(f"{name}: {t}: wanted {want[:60]!r}, got {got!r}")
    return bad


def args(p) -> None:
    p.add_argument("--jobs", type=int, default=os.cpu_count() or 4)
    p.add_argument("filter", nargs="?", default="")


def run(a) -> list[Problem]:
    board = json.loads((specgate.ROOT / "research/scoreboard.json").read_text())
    cases = [k for k, v in board["suites"]["conformance"]["cases"].items() if v == "pass" and a.filter in k]
    with ThreadPoolExecutor(a.jobs) as ex:
        found = [b for bs in ex.map(check, cases) for b in bs]
    return [Problem(*b.split(": ", 1)) for b in found]
