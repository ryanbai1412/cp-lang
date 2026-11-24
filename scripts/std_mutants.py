"""Mutation testing over std/*.cp: plant one small bug in std, rerun std's own suites.

A mutant is one edit at one std code line: a comparison loosened or flipped, `and`
for `or`, `+` for `-`, `true` for `false`, a `not` dropped. It is written to a scratch
copy of std/ that `CPL_STD_DIR` points the tools at, so no rebuild is needed. The
suites closest to the edit run first and the run stops at the first case that passed
on the clean std and now fails (caught): the file's own `@test` fns, the doc examples of the enclosing item, the
file's `spec/conformance/std-<file>/` cases, the `spec/conformance/std-models/` cases
naming a type of the file or the item, then every doc example of the file. A mutant
whose newly failing cases all fail to compile is unviable; one past its time limit
timed out; neither is scored. Driven by `scripts/mutation-score.py --std`.
"""
from __future__ import annotations

import json
import os
import queue
import re
import shutil
import subprocess
import tempfile
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
STD = ROOT / "std"
MODELS = ROOT / "spec/conformance/std-models"

OPS = [
    (re.compile(r" < "), " <= "),
    (re.compile(r" <= "), " < "),
    (re.compile(r" > "), " >= "),
    (re.compile(r" >= "), " > "),
    (re.compile(r" == "), " != "),
    (re.compile(r" != "), " == "),
    (re.compile(r" and "), " or "),
    (re.compile(r" or "), " and "),
    (re.compile(r" \+ "), " - "),
    (re.compile(r" - "), " + "),
    (re.compile(r" \* "), " + "),
    (re.compile(r" \+= "), " -= "),
    (re.compile(r" -= "), " += "),
    (re.compile(r"\btrue\b"), "false"),
    (re.compile(r"\bfalse\b"), "true"),
    (re.compile(r"\bnot "), ""),
]
HEADER = re.compile(r"^\s*(?:pub\s+)?(?:@\w+(?:\([^)]*\))?\s+)*(?:pub\s+)?(?:fn|struct|trait|impl|enum|type|use|const)\b")
ITEM = re.compile(r"^(?:pub\s+)?(?:@\w+(?:\([^)]*\))?\s+)*(?:pub\s+)?"
                  r"(?:fn|struct|trait|enum|const)\s+([A-Za-z_]\w*)|^impl(?:<[^>]*>)?\s+(?:[\w<>, ]+\s+for\s+)?([A-Za-z_]\w*)")
TYPE = re.compile(r"^pub\s+(?:struct|enum|trait)\s+([A-Za-z_]\w*)", re.M)


def masked(line: str) -> str:
    """The code part of a line: strings blanked to `_`, a trailing `//` comment cut."""
    out, quote, i = [], False, 0
    while i < len(line):
        c = line[i]
        if quote:
            if c == "\\":
                out.append("__")
                i += 2
                continue
            quote = c != '"'
            out.append('"' if c == '"' else "_")
        elif c == '"':
            quote = True
            out.append(c)
        elif line.startswith("//", i):
            break
        else:
            out.append(c)
        i += 1
    return "".join(out)


def mutable_span(line: str) -> tuple[int, str]:
    """(offset, text) of the part of a line an operator may change: a declaration
    line only past its body's `=`, a non-code line not at all."""
    code = masked(line)
    if not code.strip() or code.lstrip().startswith("@"):
        return 0, ""
    if HEADER.match(code):
        close = code.rfind(")")
        at = code.find(" = ", max(close, 0))
        return (at + 3, code[at + 3:]) if at >= 0 else (0, "")
    return 0, code


def line_mutants(line: str) -> list[tuple[int, str, str, str]]:
    """Every (column, was, now, mutated line) one operator makes at one place on `line`."""
    off, code = mutable_span(line)
    seen, out = set(), []
    for pat, rep in OPS:
        for m in pat.finditer(code):
            a, b = off + m.start(), off + m.end()
            new = line[:a] + rep + line[b:]
            if new != line and new not in seen:
                seen.add(new)
                out.append((a + 1 + (len(m.group(0)) - len(m.group(0).lstrip())),
                            m.group(0).strip(), rep.strip() or "nothing", new))
    return out


def item_at(lines: list[str], i: int) -> str:
    """The top-level item (fn or type) line `i` is inside."""
    for j in range(i, -1, -1):
        if lines[j] and not lines[j][0].isspace() and (m := ITEM.match(lines[j])):
            return m.group(1) or m.group(2)
    return ""


def sites(files: list[Path], only: dict[str, set[int]] | None = None) -> list[dict]:
    """Every mutant of the std files, in file and line order; `only` limits each
    file (by its std-relative name) to those 1-based lines."""
    out = []
    for path in files:
        rel = path.relative_to(STD).as_posix()
        if only is not None and rel not in only:
            continue
        lines = path.read_text(encoding="utf-8").split("\n")
        tests = test_fn_lines(lines)
        for i, line in enumerate(lines):
            if i in tests or (only is not None and i + 1 not in only[rel]):
                continue
            for col, was, now, new in line_mutants(line):
                out.append({"file": rel, "line": i + 1, "was": was, "now": now, "text": new,
                            "item": item_at(lines, i),
                            "name": f"std/{rel}:{i + 1}:{col}: replace `{was}` with `{now}` in `{line.strip()}`"})
    return out


def changed_lines(since: str, root: Path = ROOT) -> dict[str, set[int]]:
    """The std lines (1-based, in the working tree) that differ from `since`."""
    diff = subprocess.run(["git", "diff", "-U0", "--no-color", since, "--", "std/"], cwd=root,
                          capture_output=True, text=True, check=True).stdout
    return parse_diff(diff)


def parse_diff(diff: str) -> dict[str, set[int]]:
    out: dict[str, set[int]] = {}
    cur = None
    for line in diff.splitlines():
        if line.startswith("+++ "):
            cur = line[6:] if line.startswith("+++ b/std/") else None
            cur = cur[4:] if cur else None
        elif cur and (m := re.match(r"@@ -\S+ \+(\d+)(?:,(\d+))? @@", line)):
            start, n = int(m.group(1)), int(m.group(2) or 1)
            out.setdefault(cur, set()).update(range(start, start + n))
    return out


def shard(mutants: list, spec: str | None) -> list:
    if not spec:
        return mutants
    k, n = (int(x) for x in spec.split("/"))
    return mutants[k::n] if k < n else []


def test_argv(cpl: Path | str, file: str) -> list[str]:
    """`cpl test std/<file>`, run from the folder holding the std copy: only a path that
    reads `std/<file>` checks as std's own module; any other path re-imports it and clashes."""
    return [str(cpl), "test", f"std/{file}"]


def test_cases(out: str, code: int) -> dict:
    """{fn: pass|fail} from `cpl test` on a std file; {"*": error} when it did not compile."""
    cases = {m.group(2): "pass" if m.group(1) == "ok" else "fail"
             for m in re.finditer(r"^\s*(ok|FAIL)\s+\S*?::(\w+)", out, re.M)}
    if not cases and code:
        return {"*": "error" if "error[E" in out else "fail"}
    return cases


def test_fn_lines(lines: list[str]) -> set[int]:
    """0-based lines of the `@test` fns: test code, not std code, so never a mutant site."""
    out, inside = set(), False
    for i, line in enumerate(lines):
        if line.strip() == "@test" and not line[0].isspace():
            inside = True
        elif inside and line and not line[0].isspace() and not line.startswith("fn "):
            inside = False
        if inside:
            out.add(i)
    return out


class Runner:
    """Runs one suite slice against a std directory and returns {case: status}."""

    def __init__(self, conform: str, cpl: str, timeout: float = 120):
        self.conform, self.cpl, self.timeout = conform, cpl, timeout
        self.baseline: dict[tuple, dict | None] = {}

    def slices(self, m: dict) -> list[tuple]:
        stem = Path(m["file"]).stem
        out = [("test", m["file"])]
        out += [("docs", f"{m['file']}#{m['item']}")] if m["item"] else []
        if (ROOT / f"spec/conformance/std-{stem}").is_dir():
            out.append(("conformance", f"std-{stem}/"))
        out += [("model", p.name) for p in self.models(m)]
        out.append(("docs", f"{m['file']}#"))
        return out

    def models(self, m: dict) -> list[Path]:
        types = set(TYPE.findall((STD / m["file"]).read_text(encoding="utf-8")))
        names = types | ({m["item"]} if m["item"] else set())
        found = []
        for p in sorted(MODELS.glob("*.cp")):
            text = p.read_text(encoding="utf-8")
            named = set(re.findall(r"^// (?:model|brute): (\w+)", text, re.M))
            if named & names or (m["item"] and re.search(rf"\b{re.escape(m['item'])}\b", text)):
                found.append(p)
        return found

    def run(self, sl: tuple, std: Path, limit: float) -> dict | None:
        """{case: status} of one slice; None when the run crashed; "timeout" when over `limit`."""
        env = {**os.environ, "CPL_STD_DIR": str(std), "RUST_MIN_STACK": str(64 << 20)}
        kind, what = sl
        try:
            if kind == "model":
                return self.model(MODELS / what, env, limit)
            if kind == "test":
                r = subprocess.run(test_argv(self.cpl, what), cwd=std.parent, env=env,
                                   capture_output=True, text=True, timeout=limit)
                return test_cases(r.stdout + r.stderr, r.returncode)
            with tempfile.TemporaryDirectory() as tmp:
                save = Path(tmp) / "r.json"
                subprocess.run([self.conform, kind, what, "--save", str(save)], cwd=ROOT, env=env,
                               capture_output=True, timeout=limit)
                if not save.is_file():
                    return None
                return json.loads(save.read_text())["suites"][kind]["cases"]
        except subprocess.TimeoutExpired:
            return {"*": "timeout"}

    def model(self, path: Path, env: dict, limit: float) -> dict:
        out = {}
        for seed in range(1, 6):
            r = subprocess.run([self.cpl, "run", str(path)], input=f"{seed} 300\n", cwd=ROOT, env=env,
                               capture_output=True, text=True, timeout=limit)
            ok = r.returncode == 0 and r.stdout.strip() == "ok"
            out[f"{path.name}@{seed}"] = "pass" if ok else ("error" if "error[E" in r.stderr else "fail")
        return out

    def clean(self, sl: tuple) -> dict | None:
        if sl not in self.baseline:
            self.baseline[sl] = self.run(sl, STD, self.timeout)
        return self.baseline[sl]

    def outcome(self, m: dict, std: Path) -> str:
        """CaughtMutant, MissedMutant, Unviable or Timeout for mutant `m` written into `std`."""
        unviable = False
        for sl in self.slices(m):
            base = self.clean(sl)
            if not base or base.get("*") == "timeout":
                continue
            passed = {k for k, v in base.items() if v == "pass"}
            got = self.run(sl, std, self.timeout)
            if got is None:
                return "CaughtMutant"
            if got.get("*") == "timeout":
                return "Timeout"
            whole = got.get("*", "missing")
            broke = {k: got.get(k, whole) for k in passed if got.get(k) != "pass"}
            if broke and all(v == "error" for v in broke.values()):
                unviable = True
            elif broke:
                return "CaughtMutant"
        return "Unviable" if unviable else "MissedMutant"


def run_all(mutants: list[dict], runner: Runner, jobs: int, out: Path) -> dict:
    """Score every mutant; writes and returns cargo-mutants-shaped outcomes."""
    out.mkdir(parents=True, exist_ok=True)
    with ThreadPoolExecutor(max_workers=max(jobs, 1)) as pool:
        list(pool.map(runner.clean, dict.fromkeys(sl for m in mutants for sl in runner.slices(m))))
    free: queue.Queue[Path] = queue.Queue()
    for k in range(max(jobs, 1)):
        std = out / f"std-{k}" / "std"
        shutil.rmtree(std, ignore_errors=True)
        shutil.copytree(STD, std)
        free.put(std)

    def one(m: dict) -> tuple[dict, str]:
        std = free.get()
        target = std / m["file"]
        original = (STD / m["file"]).read_text(encoding="utf-8")
        lines = original.split("\n")
        target.write_text("\n".join(lines[:m["line"] - 1] + [m["text"]] + lines[m["line"]:]), encoding="utf-8")
        try:
            return m, runner.outcome(m, std)
        finally:
            target.write_text(original, encoding="utf-8")
            free.put(std)

    keys = {"CaughtMutant": "caught", "MissedMutant": "missed", "Timeout": "timeout", "Unviable": "unviable"}
    result: dict = {v: 0 for v in keys.values()} | {"outcomes": []}
    with ThreadPoolExecutor(max_workers=max(jobs, 1)) as pool:
        for m, summary in pool.map(one, mutants):
            result[keys[summary]] += 1
            result["outcomes"].append({"scenario": {"Mutant": {"name": m["name"]}}, "summary": summary})
            print(f"{summary}: {m['name']}", flush=True)
    (out / "mutants.out").mkdir(exist_ok=True)
    (out / "mutants.out" / "outcomes.json").write_text(json.dumps(result, indent=1), encoding="utf-8")
    return result
