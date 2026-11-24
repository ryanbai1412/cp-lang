#!/usr/bin/env python3
"""The corpus package: one corpus/problems/<id>/ layout, one walker.

  scripts/corpus.py build          regenerate the generated fields of every
                                   meta.json and corpus/index.json, keeping
                                   hand-written fields
  scripts/corpus.py check          fail when a meta.json is stale or names a
                                   missing file
  scripts/corpus.py verify <id>... [--cpl PATH] [--check]
                                   verify packages; records `dream_stress` in
                                   each meta.json (--check writes nothing and
                                   fails unless the dream agrees and meta.json
                                   already records that for this dream.cp;
                                   problem-packages/R10)

Every tool that walks the corpus calls `problems()` and reads a `Package`;
nothing else lists corpus/problems (handbook/guidelines/
problem-packages-checklist.md#r8).

Generated fields: problem, source, submissions, artifacts. Hand-written fields
(tags, difficulty, dream, spec, notes) and recorded ones (dream_stress)
survive a rebuild.
"""
import argparse, hashlib, importlib.util, json, os, re, subprocess, sys, tempfile, tomllib
from collections import Counter, defaultdict
from dataclasses import dataclass
from pathlib import Path
from typing import Iterator, Optional

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SUBS = os.path.join(ROOT, "corpus", "submissions")
PROBLEMS = os.path.join(ROOT, "corpus", "problems")
SETS = os.path.join(ROOT, "corpus", "sets")
CXX_CACHE = os.path.join(ROOT, "target", "corpus-cxx")
HAND = {"tags": [], "difficulty": None, "dream": "none", "spec": "none", "notes": ""}
RECORDED = ("dream_stress",)
STATUSES = {"none", "draft", "reviewed", "gold"}
EXT = {"CPP": ".cpp", "CPP03": ".cpp", "CPP11": ".cpp", "CPP14": ".cpp", "CPP17": ".cpp",
       "CPP20": ".cpp", "JAVA": ".java", "PY3": ".py", "PYPY3": ".py", "PY2": ".py", "C": ".c"}


def rel(p):
    return os.path.relpath(p, ROOT)


@dataclass(frozen=True)
class Package:
    """One problem package. Each file is its Path, or None when absent."""
    id: str
    dir: Path
    meta: Optional[Path]
    port: Optional[Path]
    spec: Optional[Path]
    dream: Optional[Path]
    orig: Optional[Path]
    gen: Optional[Path]
    checker: Optional[Path]
    interactor: Optional[Path]
    tests: Optional[Path]

    @classmethod
    def at(cls, d):
        d = Path(d)
        f = lambda name: d / name if (d / name).is_file() else None
        return cls(id=d.name, dir=d, meta=f("meta.json"), port=f("port.cp"), spec=f("spec.cp"),
                   dream=f("dream.cp"), orig=f("orig.cpp"), gen=f("gen.py"), checker=f("checker.py"),
                   interactor=f("interactor.py"), tests=f("tests.toml"))

    def read_meta(self):
        return json.loads(self.meta.read_text()) if self.meta else {}

    def reference(self):
        """The accepted C++: orig.cpp, else meta.json's first AC .cpp submission."""
        if self.orig:
            return self.orig
        for s in self.read_meta().get("submissions", []):
            f = s.get("file") or ""
            if s.get("result") == "AC" and f.endswith(".cpp") and os.path.exists(os.path.join(ROOT, f)):
                return Path(ROOT, f)
        return None

    def cases(self):
        """tests.toml's `[[case]]` tables: `input` (a statement sample), `seed` and `size`, or `args`."""
        return tomllib.loads(self.tests.read_text()).get("case", []) if self.tests else []

    def case_input(self, case, timeout=60):
        if "input" in case:
            text = case["input"]
        else:
            argv = [str(a) for a in case["args"]] if "args" in case else [str(case["seed"]), str(case["size"])]
            text = subprocess.run([sys.executable, str(self.gen), *argv], capture_output=True, text=True,
                                  timeout=timeout, check=True).stdout
        return text if text.endswith("\n") else text + "\n"


def set_ids(name):
    """The ids listed in corpus/sets/<name>.txt."""
    with open(os.path.join(SETS, name + ".txt")) as f:
        return {l.split("#")[0].strip() for l in f} - {""}


def problems(sets=None, ids=None) -> Iterator[Package]:
    """Every package under corpus/problems, sorted by id; only those in any of
    `sets` (corpus/sets/<name>.txt) and in `ids` when given."""
    keep = set(ids) if ids else None
    if sets:
        members = set().union(*(set_ids(s) for s in sets))
        keep = members if keep is None else keep & members
    if not os.path.isdir(PROBLEMS):
        return
    for name in sorted(os.listdir(PROBLEMS)):
        d = os.path.join(PROBLEMS, name)
        if os.path.isdir(d) and (keep is None or name in keep):
            yield Package.at(d)


def blob_sha(path):
    return subprocess.run(["git", "hash-object", str(path)], capture_output=True, text=True, check=True).stdout.strip()


# Accepted submissions use C++23 (deducing `this`, <ranges>), so they build with
# the clang the ziglang wheel ships on every platform, with a bits/stdc++.h shim.
ACCEPTED_CXX = [sys.executable, "-m", "ziglang", "c++", "-std=c++23", "-O2", "-w",
                "-I", os.path.join(ROOT, "bench", "cxx23-shim")]
# libstdc++'s GNU extensions (policy-based data structures, rope) exist only
# under g++, the judges' compiler.
GNU_EXT = re.compile(r"__gnu_pbds|__gnu_cxx|ext/pb_ds|ext/rope")
GNU_CXX = ["g++", "-std=gnu++2b", "-O2", "-w"]


def accepted_cxx(text):
    """The compiler command for accepted C++ `text`."""
    return GNU_CXX if GNU_EXT.search(text) else ACCEPTED_CXX


class InfraError(Exception):
    """The toolchain, a generator or the accepted C++ failed: no verdict, so nothing is recorded."""


def binary(src):
    """`src` built with ACCEPTED_CXX, cached by content under target/corpus-cxx/."""
    os.makedirs(CXX_CACHE, exist_ok=True)
    data = Path(src).read_bytes()
    key = hashlib.sha1(data).hexdigest()[:16]
    exe = os.path.join(CXX_CACHE, key)
    if not os.path.exists(exe):
        cxx = accepted_cxx(data.decode(errors="replace"))
        r = subprocess.run([*cxx, "-o", exe, str(src)], capture_output=True, text=True)
        if r.returncode and cxx is ACCEPTED_CXX:
            # The judge accepted it under g++: GNU dialect clang rejects
            # (`int a[n] {};`) is still the reference.
            r = subprocess.run([*GNU_CXX, "-o", exe, str(src)], capture_output=True, text=True)
        if r.returncode:
            raise RuntimeError("c++: " + r.stderr[:200])
    return exe


def judge_stack():
    """The judges' stack, so a deep recursion in the accepted C++ runs as it did there."""
    import resource
    resource.setrlimit(resource.RLIMIT_STACK, (resource.RLIM_INFINITY, resource.RLIM_INFINITY))


STRESS_SIZES = (4, 8, 16, 32)


def stress_inputs(pkg):
    """tests.toml's cases, else gen.py at small sizes: what a solution must agree on."""
    if pkg.tests:
        return [pkg.case_input(c) for c in pkg.cases()]
    if pkg.gen:
        return [pkg.case_input({"seed": i, "size": s}) for i, s in enumerate(STRESS_SIZES * 2, 1)]
    return []


def package_limits(pkg):
    """The problem's limits, from its port's `// limits:` line, so a dream is judged as the port is."""
    if not pkg.port:
        return []
    spec = importlib.util.spec_from_file_location("generators", os.path.join(ROOT, "corpus", "generators.py"))
    gens = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(gens)
    ms, mb = gens.limits(pkg.port)
    return ["--time-limit", str(ms), "--memory-limit", str(mb)]


def judged(pkg, source, inputs, cpl):
    """Run `source` with `cpl test` on `inputs`, answers from the accepted C++
    (none for an interactive package): (all passed, cases run)."""
    ref = None if pkg.interactor else pkg.reference()
    if not inputs or (ref is None and not pkg.interactor):
        return False, 0
    with tempfile.TemporaryDirectory() as cases:
        try:
            exe = binary(ref) if ref else None
        except RuntimeError as e:
            raise InfraError(f"{pkg.id}: {e}") from e
        for i, text in enumerate(inputs, 1):
            Path(cases, f"{i:02}.in").write_text(text)
            if exe:
                r = subprocess.run([exe], input=text, capture_output=True, text=True, timeout=60, preexec_fn=judge_stack)
                if r.returncode:
                    raise InfraError(f"{pkg.id}: accepted C++ exits {r.returncode} on case {i}")
                Path(cases, f"{i:02}.ans").write_text(r.stdout)
        judge = ["--interactor", str(pkg.interactor)] if pkg.interactor else \
                ["--checker", str(pkg.checker)] if pkg.checker else []
        r = subprocess.run([cpl, "test", str(source), "--tests", cases, "--format", "json", *judge, *package_limits(pkg)],
                           cwd=ROOT, capture_output=True, text=True)
        try:
            results = json.loads(r.stdout)["cases"]
        except (ValueError, KeyError):
            if "does not build" in r.stderr:
                return False, 0
            raise InfraError(f"{pkg.id}: cpl test: {r.stderr.strip()[:300]}")
        return bool(results) and all(c["verdict"] == "ok" for c in results), len(results)


def dream_stress(pkg, cpl):
    """Does dream.cp agree with the accepted C++ on the package's inputs?"""
    try:
        ok, n = judged(pkg, pkg.dream, stress_inputs(pkg), cpl)
    except (RuntimeError, subprocess.SubprocessError, OSError) as e:
        raise InfraError(f"{pkg.id}: {e}") from e
    return {"ok": ok, "cases": n, "sha": blob_sha(pkg.dream)}


def record(pkg, field, value):
    meta = pkg.read_meta()
    meta[field] = value
    pkg.meta.write_text(dump(meta))


def verify(args):
    cpl = args.cpl or os.path.join(ROOT, "target", "release", "cpl")
    wanted = list(problems(ids=args.ids))
    missing = set(args.ids) - {p.id for p in wanted}
    if missing:
        sys.exit(f"corpus: no package {', '.join(sorted(missing))}")
    failed = 0
    for pkg in wanted:
        if pkg.dream and pkg.meta:
            try:
                got = dream_stress(pkg, cpl)
            except InfraError as e:
                print(f"corpus: no verdict, the toolchain failed: {e}", file=sys.stderr)
                return 2
            print(f"{pkg.id}: dream-stress {'ok' if got['ok'] else 'FAILS'} on {got['cases']} case(s)")
            failed += not got["ok"]
            if not args.check:
                record(pkg, "dream_stress", got)
            elif got["ok"] and (pkg.read_meta().get("dream_stress") or {}).get("sha") != got["sha"]:
                print(f"{pkg.id}: meta.json does not record this dream.cp's agreement; "
                      f"run scripts/corpus.py verify {pkg.id} and stage meta.json", file=sys.stderr)
                failed += 1
    return 1 if failed else 0


def canonical_id(problem):
    if re.fullmatch(r"dp[a-z]", problem):
        return f"atcoder_dp_{problem[2]}"
    return problem


def source_of(problem):
    if problem.startswith("atcoder_dp_"):
        return "atcoder_dp"
    m = re.match(r"[a-z]+", problem)
    return m.group(0) if m else problem


def artifacts():
    found = defaultdict(lambda: defaultdict(list))
    gold_dir = os.path.join(ROOT, "corpus", "gold")
    if os.path.isdir(gold_dir):
        for d, _, files in os.walk(gold_dir):
            for f in files:
                m = re.match(r"([a-z0-9_]+?)\.(cp)$", f)
                if m:
                    found[m.group(1)]["dream"].append(rel(os.path.join(d, f)))
    for pkg in problems():
        for kind, f in (("dream", pkg.dream), ("spec", pkg.spec), ("port", pkg.port), ("original", pkg.orig)):
            if f:
                found[pkg.id][kind].insert(0, rel(f))
    return found


def generate():
    info = json.load(open(os.path.join(SUBS, "info.json")))
    by = defaultdict(list)
    for sid, e in info.items():
        by[canonical_id(e["problem"])].append((sid, e))
    arts = artifacts()
    all_problems = sorted(set(by.keys()) | set(arts.keys()))
    out = {}
    for problem in all_problems:
        subs = by.get(problem, [])
        rows = []
        for sid, e in sorted(subs, key=lambda x: x[1].get("date", "")):
            f = next((sid + x for x in (EXT.get(e.get("language", ""), ""), ".cpp", ".java", ".py")
                      if x and os.path.exists(os.path.join(SUBS, sid + x))), None)
            rows.append({"id": sid, "language": e.get("language"), "result": e.get("result"),
                         "time": e.get("time"), "memory_kb": e.get("memory"),
                         "file": rel(os.path.join(SUBS, f)) if f else None})
        p_arts = {k: sorted(v) for k, v in sorted(arts.get(problem, {}).items())}
        cpp_ac = sum(1 for r in rows if r["result"] == "AC" and (r["file"] or "").endswith(".cpp"))
        if not cpp_ac and p_arts.get("original"):
            cpp_ac = len(p_arts["original"])
        out[problem] = {
            "problem": problem,
            "source": source_of(problem),
            "submissions": rows,
            "cpp_ac": cpp_ac,
            "artifacts": p_arts,
        }
    return out


def merged(gen, path):
    old = json.load(open(path)) if os.path.exists(path) else {}
    meta = dict(gen)
    for k, v in HAND.items():
        meta[k] = old.get(k, v)
    for k in RECORDED:
        if k in old:
            meta[k] = old[k]
    return meta


def dump(x):
    text = json.dumps(x, indent=1)
    return re.sub(r"\{\n\s+(\"id\"[^{}]*?)\n\s+\}", lambda m: "{" + re.sub(r"\n\s+", " ", m.group(1)) + "}", text) + "\n"


def index(metas):
    return {"problems": len(metas),
            "with_cpp_ac": sum(1 for m in metas.values() if m["cpp_ac"]),
            "dream": {s: sum(1 for m in metas.values() if m["dream"] == s) for s in sorted(STATUSES)},
            "spec": {s: sum(1 for m in metas.values() if m["spec"] == s) for s in sorted(STATUSES)},
            "by_source": dict(sorted(Counter(m["source"] for m in metas.values()).items()))}


def main(mode):
    gen = generate()
    metas, errors = {}, []
    for problem, g in gen.items():
        path = os.path.join(PROBLEMS, problem, "meta.json")
        meta = merged(g, path)
        metas[problem] = meta
        for k in ("dream", "spec"):
            if meta[k] not in STATUSES:
                errors.append(f"{rel(path)}: {k} must be one of {sorted(STATUSES)}")
        for f in [r["file"] for r in meta["submissions"] if r["file"]] + \
                 [f for v in meta["artifacts"].values() for f in v]:
            if not os.path.exists(os.path.join(ROOT, f)):
                errors.append(f"{rel(path)}: missing {f}")
        if mode == "build":
            os.makedirs(os.path.dirname(path), exist_ok=True)
            open(path, "w").write(dump(meta))
        elif not os.path.exists(path) or open(path).read() != dump(meta):
            errors.append(f"{rel(path)}: stale; run scripts/corpus.py build")
    for pkg in problems():
        if pkg.id not in gen:
            errors.append(f"corpus/problems/{pkg.id}: no submissions or artifacts")
    ipath = os.path.join(ROOT, "corpus", "index.json")
    if mode == "build":
        open(ipath, "w").write(dump(index(metas)))
    elif not os.path.exists(ipath) or open(ipath).read() != dump(index(metas)):
        errors.append("corpus/index.json: stale; run scripts/corpus.py build")
    for e in errors:
        print(e, file=sys.stderr)
    print(f"corpus: {len(metas)} problems, {len(errors)} error(s)")
    return 1 if errors else 0


def cli(argv):
    ap = argparse.ArgumentParser(usage=__doc__)
    sub = ap.add_subparsers(dest="mode", required=True)
    sub.add_parser("build")
    sub.add_parser("check")
    v = sub.add_parser("verify")
    v.add_argument("ids", nargs="+")
    v.add_argument("--cpl")
    v.add_argument("--check", action="store_true")
    args = ap.parse_args(argv)
    return verify(args) if args.mode == "verify" else main(args.mode)


def toolchain(argv):
    """Modes that build C++ need ACCEPTED_CXX's ziglang wheel: run under the repo's
    .venv when this python lacks it, else stop and name it."""
    if not argv or argv[0] != "verify" or importlib.util.find_spec("ziglang"):
        return
    venv = Path(ROOT, ".venv", "bin", "python")
    if venv.exists() and Path(sys.prefix).resolve() != venv.parent.parent.resolve():
        os.execv(str(venv), [str(venv), __file__, *argv])
    sys.exit("corpus.py: this python has no ziglang wheel (ACCEPTED_CXX); run `uv sync`, "
             "or `uv run python scripts/corpus.py`")


if __name__ == "__main__":
    toolchain(sys.argv[1:])
    sys.exit(cli(sys.argv[1:]))
