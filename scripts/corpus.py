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
import argparse, functools, hashlib, importlib.util, json, os, re, shutil, signal, subprocess, sys, tempfile, threading, time, tomllib
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


# Accepted submissions were judged by g++: C++23 (deducing `this`, <ranges>), GNU
# dialect (`int a[n] {};`) and libstdc++'s extensions (pb_ds, rope). So references
# build with one compiler, g++ 14 or newer (deducing `this` needs 14), statically linked so a toolchain
# outside the system's (conda-forge's on the dev VM) runs them too.
REFERENCE_GXX = ("g++-15", "g++-14", "g++")
REFERENCE_FLAGS = ["-std=gnu++23", "-O2", "-w", "-static-libstdc++", "-static-libgcc"]


@functools.cache
def reference_cxx():
    """The g++ 14+ command that builds accepted C++, and its version."""
    def version_14(path):
        v = subprocess.run([path, "-dumpfullversion"], capture_output=True, text=True).stdout.strip()
        return v if v and int(v.split(".")[0]) >= 14 else None
    for path in filter(None, map(shutil.which, REFERENCE_GXX)):
        if version := version_14(path):
            return [path, *REFERENCE_FLAGS], version
    if os.environ.get("CPL_GXX_FETCH", "1") != "0":
        fetched = subprocess.run(["bash", os.path.join(ROOT, "scripts", "gxx14.sh")],
                                 stdout=subprocess.PIPE, text=True)
        path = fetched.stdout.strip()
        if fetched.returncode == 0 and (version := version_14(path)):
            return [path, *REFERENCE_FLAGS], version
    raise RuntimeError(f"no g++ 14 or newer on PATH (tried {', '.join(REFERENCE_GXX)}) "
                       "and scripts/gxx14.sh could not fetch one; "
                       "accepted C++ needs the judges' C++23 g++ (README: Setup)")


class InfraError(Exception):
    """The toolchain, a generator or the accepted C++ failed: no verdict, so nothing is recorded."""


def binary(src):
    """`src` built with reference_cxx(), cached by content and compiler under target/corpus-cxx/."""
    os.makedirs(CXX_CACHE, exist_ok=True)
    cxx, version = reference_cxx()
    data = Path(src).read_bytes()
    key = hashlib.sha1(data + b"\0gcc " + version.encode()).hexdigest()[:16]
    exe = os.path.join(CXX_CACHE, key)
    if not os.path.exists(exe):
        r = subprocess.run([*cxx, "-o", exe, str(src)], capture_output=True, text=True)
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
    ms, mb = limits(pkg.port)
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


DREAM_ERROR = re.compile(r"^error\[(E\d{4})\]: (.*)$")


def dream_errors(cpl, path):
    """(code, message, help) for each error `cpl check` prints, in order."""
    out = subprocess.run([cpl, "check", str(path)], capture_output=True, text=True).stderr
    found = []
    for line in out.splitlines():
        m = DREAM_ERROR.match(line)
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
        errs = dream_errors(cpl, path)
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
        lines.append("  … %d more (scripts/corpus.py gaps)" % (len(r["gaps"]) - top))
    return lines


def record(pkg, field, value):
    meta = pkg.read_meta()
    meta[field] = value
    pkg.meta.write_text(dump(meta))


HEADER_FIELDS = ("Source", "Problem", "Solution")
HEADER_LINE = re.compile(r"//\s*(Source|Problem|Solution|Bounds):")
TIMEOUT = 20
SLOWER, FLOOR = 1.5, 0.1
TARGET_BYTES = 32 * 1024
GEN_SEEDS, GEN_SIZES = (1, 2, 3), (5, 50)
RANGE = re.compile(rb"error\[R05\d\d\]|outside its range")
MAIN = re.compile(r"\bmain\s*\(")
MODES = ("header", "check", "package", "tests", "stress", "max", "size", "lint", "dream")
# The modes that run a port: only the judge-proven ports of corpus/sets/v03.txt.
RUN_SET = "v03"


def header_fields(text):
    """The `// Field:` names in the comment block a file opens with."""
    found = set()
    for line in text.split("\n"):
        if not line.startswith("//"):
            break
        m = HEADER_LINE.match(line)
        if m:
            found.add(m.group(1))
    return found


def check_header(pkg, ctx):
    if not pkg.port:
        return []
    have = header_fields(pkg.port.read_text(errors="replace"))
    return [f"no-header-{f.lower()} {pkg.id} -" for f in HEADER_FIELDS if f not in have]


def check_check(pkg, ctx):
    """`cpl check port.cp` reports no error (a warning never fails)."""
    if not pkg.port:
        return []
    r = subprocess.run([ctx.cpl, "check", str(pkg.port)], capture_output=True, text=True)
    if not re.search(r"(?m)^error", r.stderr):
        return []
    # The log is the only evidence a CI runner keeps: show the port's errors.
    print(f"corpus: {ctx.cpl} check {os.path.relpath(pkg.port, ROOT)}", file=sys.stderr)
    print("\n".join(r.stderr.splitlines()[:40]), file=sys.stderr)
    return [f"port-check {pkg.id} -"]


def tokens(data):
    return data.split()


def run_out(exe, data):
    try:
        r = subprocess.run(exe if isinstance(exe, list) else [exe], input=data, capture_output=True,
                           timeout=TIMEOUT, preexec_fn=judge_stack)
    except subprocess.TimeoutExpired:
        return None
    return r.stdout if r.returncode == 0 else None


def accepted_cpps(pkg):
    out = [pkg.orig] if pkg.orig else []
    for s in pkg.read_meta().get("submissions", []):
        f = s.get("file") or ""
        p = Path(ROOT, f)
        if s.get("result") == "AC" and f.endswith(".cpp") and p.exists() and p not in out:
            out.append(p)
    return out


def built(src):
    try:
        return binary(src)
    except RuntimeError:
        return None


def check_package(pkg, ctx):
    """problem-packages R1-R4: answers from an accepted C++, a checker where
    answers vary, a generator for interactive hidden data, a logged dream."""
    found = []
    if pkg.interactor and not pkg.gen:
        found.append(f"interactor-no-gen {pkg.id} -")
    if pkg.dir.joinpath("tests").exists():
        found.append(f"committed-tests {pkg.id} -")
    if pkg.dream and pkg.spec:
        code = lambda p: [l.split("//")[0].rstrip() for l in p.read_text(errors="replace").splitlines() if l.split("//")[0].strip()]
        if code(pkg.dream) != code(pkg.spec) and not (pkg.read_meta().get("notes") or "").strip() \
                and not re.search(rf"\b{re.escape(pkg.id)}\b", ctx.hillclimb()):
            found.append(f"dream-unlogged {pkg.id} -")
    found += [f"stale-tests-path {pkg.id} {path}" for path in ctx.tests_paths().get(pkg.id, ())]
    for inp in generated(pkg) if pkg.tests else ():
        ans = inp.with_suffix(".ans")
        t = tokens(inp.read_bytes())
        if ans.exists() and len(t) > 2 and t == tokens(ans.read_bytes()):
            found.append(f"answer-is-input {pkg.id} {inp.name}")
    if not pkg.tests:
        return found
    if not pkg.interactor:
        srcs = accepted_cpps(pkg)
        if not any(MAIN.search(s.read_text(errors="replace")) for s in srcs):
            return found + [f"no-reference {pkg.id} -"]
        if ctx.run and not pkg.checker:
            exes = [e for e in map(built, srcs) if e]
            for i, case in enumerate(pkg.cases(), 1) if len(exes) > 1 else ():
                data = pkg.case_input(case).encode()
                outs = {tuple(tokens(o)) for o in (run_out(e, data) for e in exes) if o is not None}
                if len(outs) > 1:
                    found.append(f"needs-checker {pkg.id} {i}")
                    break
    return found


def generated(pkg):
    """The cases `make` wrote for the package: target/corpus-tests/<id>/*.in."""
    d = Path(ROOT, "target", "corpus-tests", pkg.id)
    return sorted(d.glob("*.in")) if d.is_dir() else []


def cpl_cases(cpl, program, *args):
    """`cpl test <program> <args>`'s cases, or the finding kind when it gives none."""
    try:
        r = subprocess.run([cpl, "test", str(program), *args, "--format", "json"], cwd=ROOT, capture_output=True,
                           text=True, timeout=10 * TIMEOUT)
    except subprocess.TimeoutExpired:
        return "cpl-test-timeout"
    if "has no cases" in r.stderr:
        return "cpl-test-no-cases"
    try:
        return json.loads(r.stdout)["cases"]
    except (ValueError, KeyError):
        return "cpl-test-error"


def check_tests(pkg, ctx):
    """`cpl test port.cp` reads the package's cases and passes every one (problem-packages/R8)."""
    if not pkg.port or not ctx.runs(pkg):
        return []
    cases = cpl_cases(ctx.cpl, pkg.port)
    if isinstance(cases, str):
        return [f"{cases} {pkg.id} -"]
    found = [f"cpl-test-fails {pkg.id} {c['name']}" for c in cases if c["verdict"] not in (None, "ok")]
    if any(c["verdict"] is None for c in cases):
        found.append(f"cpl-test-unjudged {pkg.id} -")
    return found


def native(pkg, ctx):
    """The port's Codeforces C++ built with g++ -O2 (cached): (exe, bytes), or (None, 0)."""
    if pkg.id in ctx.natives:
        return ctx.natives[pkg.id]
    with tempfile.TemporaryDirectory() as tmp:
        cpp = os.path.join(tmp, f"{pkg.id}.cpp")
        r = subprocess.run([ctx.cpl, "build", "--judge", "codeforces", str(pkg.port), "-o", cpp], capture_output=True)
        got = (None, 0) if r.returncode else (built(cpp), os.path.getsize(cpp))
    ctx.natives[pkg.id] = got
    return got


def agrees(pkg, data, got, want):
    if not pkg.checker:
        return tokens(got) == tokens(want)
    with tempfile.TemporaryDirectory() as tmp:
        files = []
        for name, blob in (("in", data), ("out", got), ("ans", want)):
            files.append(os.path.join(tmp, name))
            Path(files[-1]).write_bytes(blob)
        return subprocess.run([sys.executable, str(pkg.checker), *files], capture_output=True).returncode == 0


def rejects(cmd, data):
    try:
        r = subprocess.run(cmd, input=data, capture_output=True, timeout=TIMEOUT)
    except subprocess.TimeoutExpired:
        return False
    return r.returncode != 0 and bool(RANGE.search(r.stderr))


def gen_out(pkg, args):
    try:
        timeout = MAX_GEN_TIMEOUT if "--max" in args else TIMEOUT
        return pkg.case_input({"args": list(args)}, timeout=timeout).encode()
    except (subprocess.SubprocessError, OSError):
        return None


def run_result(cmd, data):
    """(exit code, stdout, stderr) of `cmd` on `data`, or None on a timeout."""
    try:
        r = subprocess.run(cmd, input=data, capture_output=True, timeout=TIMEOUT)
    except subprocess.TimeoutExpired:
        return None
    return r.returncode, r.stdout, r.stderr


def compare(run, interp, cxx, data):
    """Why `cpl run` and the port's C++ disagree on `data`, or None when they
    agree or the interpreter times out. A C++ timeout is retried once."""
    a = run(interp, data)
    if a is None:
        return None
    b = run(cxx, data) or run(cxx, data)
    if b is None:
        return "c++ timed out twice"
    if a[:2] == b[:2]:
        return None
    why = f"interp exit {a[0]}, c++ exit {b[0]}"
    for side, r in (("interp", a), ("c++", b)):
        if r[0] and r[2]:
            why += f", {side} stderr {b' | '.join(l.strip() for l in r[2].splitlines()[:2])!r}"
    for n, (x, y) in enumerate(zip(a[1].split(b"\n"), b[1].split(b"\n")), 1):
        if x != y:
            return why + f"; stdout line {n}: interp {x!r}, c++ {y!r}"
    return why


def check_stress(pkg, ctx):
    """The interpreter and the port's C++ agree byte for byte on every case
    (goal 5), and on gen.py's inputs the port accepts them and agrees with the
    accepted C++ (problem-packages/R5)."""
    if not pkg.port or not ctx.runs(pkg):
        return []
    if pkg.interactor:
        return interacted(pkg, ctx)
    exe, _ = native(pkg, ctx)
    if not exe:
        return [f"no-build {pkg.id} -"]
    found = []
    for inp in generated(pkg):
        why = compare(run_result, [ctx.cpl, "run", str(pkg.port)], [exe], inp.read_bytes())
        if why:
            found.append(f"differ {pkg.id} {inp.name}: {why}")
    ref = pkg.reference()
    orig = built(ref) if ref else None
    for seed in GEN_SEEDS if pkg.gen else ():
        for size in GEN_SIZES:
            data = gen_out(pkg, (seed, size))
            if data is None:
                found.append(f"gen-fails {pkg.id} {seed}/{size}")
            elif rejects([ctx.cpl, "run", str(pkg.port)], data):
                found.append(f"invalid-input {pkg.id} {seed}/{size}")
            elif orig:
                got, want = run_out(exe, data), run_out(orig, data)
                if got is not None and want is not None and not agrees(pkg, data, got, want):
                    found.append(f"disagrees {pkg.id} {seed}/{size}")
    return found


def has_max(pkg):
    return bool(pkg.gen) and "--max" in pkg.gen.read_text(errors="replace")


CHECKED_SLOWER, MAX_GEN_TIMEOUT = 10, 120
LIMITS = re.compile(r"^//\s*limits:\s*(\d+)\s*ms,\s*(\d+)\s*MB", re.M)
MAX_GEN_TIMEOUT = 120
CHECKED_SLOWER = 10


def limits(port):
    """The port's `// limits: T ms, M MB` line (what `cpl bench` reads), else the judge default."""
    m = LIMITS.search(open(port).read())
    return (int(m.group(1)), int(m.group(2))) if m else (2000, 256)


# Runs argv[2:] with the judges' stack (Codeforces gives the whole memory
# limit, so deep recursion at the max is legal) and writes its peak RSS to
# argv[1]. A child's ru_maxrss starts at its parent's RSS before exec, so the
# program must fork from this small process, not from the gate holding inputs.
LAUNCHER = r"""
#include <signal.h>
#include <stdio.h>
#include <sys/resource.h>
#include <sys/wait.h>
#include <unistd.h>
int main(int argc, char **argv) {
    struct rlimit r;
    getrlimit(RLIMIT_STACK, &r);
    r.rlim_cur = r.rlim_max;
    setrlimit(RLIMIT_STACK, &r);
    pid_t p = fork();
    if (p < 0) return 125;
    if (p == 0) { execv(argv[2], argv + 2); _exit(127); }
    int st;
    struct rusage u;
    if (wait4(p, &st, 0, &u) < 0) return 125;
    FILE *f = fopen(argv[1], "w");
    if (f) { fprintf(f, "%ld\n", (long)u.ru_maxrss); fclose(f); }
    if (WIFSIGNALED(st)) { signal(WTERMSIG(st), SIG_DFL); kill(getpid(), WTERMSIG(st)); }
    return WEXITSTATUS(st);
}
"""


def launcher():
    """The measuring launcher, built once with g++ and cached by source."""
    exe = os.path.join(ROOT, "target", "corpus-cxx", "measure-" + hashlib.sha256(LAUNCHER.encode()).hexdigest()[:16])
    if not os.path.exists(exe):
        os.makedirs(os.path.dirname(exe), exist_ok=True)
        src, tmp = f"{exe}.{os.getpid()}.{threading.get_ident()}.cpp", f"{exe}.{os.getpid()}.{threading.get_ident()}"
        with open(src, "w") as f:
            f.write(LAUNCHER)
        subprocess.run(["g++", "-O2", "-o", tmp, src], check=True, capture_output=True)
        os.remove(src)
        os.replace(tmp, exe)
    return exe


def measure(exe, data, timeout):
    """(exit code, stdout, seconds, peak MB) of one run, or None on a timeout."""
    with tempfile.TemporaryFile() as inp, tempfile.TemporaryFile() as out, \
            tempfile.NamedTemporaryFile("r") as peak:
        inp.write(data)
        inp.seek(0)
        start = time.perf_counter()
        p = subprocess.Popen([launcher(), peak.name, os.path.abspath(exe)], stdin=inp, stdout=out,
                             stderr=subprocess.DEVNULL, start_new_session=True)
        try:
            code = p.wait(timeout)
        except subprocess.TimeoutExpired:
            os.killpg(p.pid, signal.SIGKILL)
            p.wait()
            return None
        took = time.perf_counter() - start
        out.seek(0)
        kb = int(peak.read() or 0) // (1024 if sys.platform == "darwin" else 1)
        return code, out.read(), took, kb / 1024


def checked(pid, port, cpl):
    """The port's C++ with `--checks all`, built with g++ -O2 and cached by source; None if a step fails."""
    cache = os.path.join(ROOT, "target", "corpus-generators")
    os.makedirs(cache, exist_ok=True)
    cpp = os.path.join(cache, f"{pid}.{os.getpid()}.{threading.get_ident()}.cpp")
    try:
        if subprocess.run([cpl, "build", "--judge", "codeforces", "--checks", "all", port, "-o", cpp], capture_output=True).returncode:
            return None
        exe = os.path.join(cache, hashlib.sha256(open(cpp, "rb").read()).hexdigest()[:16])
        if not os.path.exists(exe):
            tmp = f"{exe}.{os.getpid()}.{threading.get_ident()}.tmp"
            if subprocess.run(["g++", "-std=c++17", "-O2", "-o", tmp, cpp], capture_output=True).returncode:
                return None
            os.replace(tmp, exe)
        return exe
    finally:
        if os.path.exists(cpp):
            os.remove(cpp)


def at_max(pkg, cpl, exe, orig, data):
    """The findings of the max input: it passes the checked C++, fits the limits, matches the accepted C++."""
    pid, port = pkg.id, str(pkg.port)
    ms, mb = limits(port)
    found = []
    safe = checked(pid, port, cpl)
    if not safe:
        found.append(f"no-build {pid} -")
    else:
        r = measure(safe, data, max(TIMEOUT, CHECKED_SLOWER * ms / 1000))
        if r is None or r[0] != 0:
            found.append(f"traps {pid} --max")
    if not exe:
        return found
    runs = [measure(exe, data, 2 * ms / 1000 + 1) for _ in range(3)]
    if any(r is None or r[0] != 0 for r in runs):
        return found + [f"over-limit {pid} --max" if any(r is None for r in runs) else f"traps {pid} --max"]
    if min(r[2] for r in runs) > ms / 1000 or min(r[3] for r in runs) > mb:
        found.append(f"over-limit {pid} --max")
    if orig:
        wants = [measure(orig, data, 4 * ms / 1000 + 1) for _ in range(3)]
        if all(w is not None and w[0] == 0 for w in wants):
            if not agrees(pkg, data, runs[0][1], wants[0][1]):
                found.append(f"disagrees {pid} --max")
            mine, theirs = min(r[2] for r in runs), min(w[2] for w in wants)
            if mine >= FLOOR and mine > SLOWER * max(theirs, FLOOR):
                found.append(f"slow {pid} --max")
    return found



def interacted(pkg, ctx):
    """The interactor rules the same on the interpreter and on the port's C++ for
    every case. An interactive case is no stdin to feed both sides: interactor.py
    talks to each, once per case, through `cpl test --interactor`."""
    cases = generated(pkg)
    if not cases:
        return []
    judge = ("--tests", str(cases[0].parent), "--interactor", str(pkg.interactor))
    ours = cpl_cases(ctx.cpl, pkg.port, *judge)
    if isinstance(ours, str):
        return [f"{ours} {pkg.id} -"]
    with tempfile.TemporaryDirectory() as tmp:
        cpp = os.path.join(tmp, f"{pkg.id}.cpp")
        if subprocess.run([ctx.cpl, "build", "--judge", "codeforces", str(pkg.port), "-o", cpp],
                          capture_output=True).returncode:
            return [f"no-build {pkg.id} -"]
        theirs = cpl_cases(ctx.cpl, cpp, *judge)
    if isinstance(theirs, str):
        return [f"differ {pkg.id} -: c++ {theirs}"]
    a, b = ({c["name"]: c for c in side} for side in (ours, theirs))
    found = []
    for inp in cases:
        x, y = a.get(inp.stem, {}), b.get(inp.stem, {})
        if x.get("verdict") != y.get("verdict"):
            found.append(f"differ {pkg.id} {inp.name}: interp {x.get('verdict')} ({x.get('why')}), "
                         f"c++ {y.get('verdict')} ({y.get('why')})")
    return found


def check_max(pkg, ctx):
    """gen.py --max makes a valid maximum input that the port's checked C++ runs
    without a trap, its judge build runs within the `// limits:` time and memory,
    and it matches the accepted C++ within 1.5x its time (problem-packages/R6).
    A missing --max is counted by the `corpus_no_max` ratchet, not here."""
    if not pkg.port or not ctx.runs(pkg) or pkg.interactor or not has_max(pkg):
        return []
    data = gen_out(pkg, ("--max",))
    if data is None:
        return [f"gen-fails {pkg.id} --max"]
    exe, _ = native(pkg, ctx)
    if exe and rejects([exe], data):
        return [f"invalid-input {pkg.id} --max"]
    ref = pkg.reference()
    orig = built(ref) if ref else None
    return at_max(pkg, ctx.cpl, exe, orig, data)


def check_size(pkg, ctx):
    """The Codeforces build fits the 32 KB every solution should (goal 5)."""
    if not pkg.port or not ctx.runs(pkg):
        return []
    _, size = native(pkg, ctx)
    return [f"over-32kb {pkg.id} {size}"] if size > TARGET_BYTES else []


def check_lint(pkg, ctx):
    """No removed spelling `cpl migrate --check` would rewrite (idiomatic-code/R8)."""
    files = [str(f) for f in (pkg.port, pkg.dream, pkg.spec) if f]
    if not files:
        return []
    r = subprocess.run([ctx.cpl, "migrate", "--check", "--json", *files], capture_output=True, text=True)
    try:
        out = json.loads(r.stdout)["files"]
    except (ValueError, KeyError):
        return [f"migrate-error {pkg.id} -"]
    return sorted(f"removed-spelling {pkg.id} {Path(f['path']).name}" for f in out if f["changed"] or f["diagnostics"])


def check_dream(pkg, ctx):
    """dream.cp agrees with the accepted C++ (problem-packages/R10). Records the
    verdict in meta.json; with `check`, writes nothing and also fails when meta.json
    doesn't record this dream.cp's agreement."""
    if not (pkg.dream and pkg.meta and ctx.run):
        return []
    got = dream_stress(pkg, ctx.cpl)
    if not got["ok"]:
        return [f"dream-stress {pkg.id} fails-on-{got['cases']}-cases"]
    if not ctx.check:
        record(pkg, "dream_stress", got)
    elif (pkg.read_meta().get("dream_stress") or {}).get("sha") != got["sha"]:
        return [f"dream-unrecorded {pkg.id} run-scripts/corpus.py-verify-{pkg.id}-and-stage-meta.json"]
    return []


CHECKS = {"header": check_header, "check": check_check, "package": check_package, "tests": check_tests,
          "stress": check_stress, "max": check_max, "size": check_size, "lint": check_lint, "dream": check_dream}


class Context:
    def __init__(self, cpl, run=True, check=False, named=()):
        self.check, self.named = check, set(named)
        self.cpl, self.run, self.natives, self._hill, self._tests = cpl, run, {}, None, None
        self.run_ids = set_ids(RUN_SET) if os.path.exists(os.path.join(SETS, RUN_SET + ".txt")) else set()

    def runs(self, pkg):
        """Runs the package's programs: it's in the run set, or named on the command line."""
        return self.run and (pkg.id in self.run_ids or pkg.id in self.named)

    def tests_paths(self):
        """id -> tracked files outside corpus/ that name corpus/problems/<id>/tests,
        a folder no package has (problem-packages/R9)."""
        if self._tests is None:
            out = subprocess.run(["git", "grep", "-l", "-E", r"corpus/problems/[A-Za-z0-9_-]+/tests\b", "--",
                                  ".", ":!corpus/", ":!research/"], cwd=ROOT, capture_output=True, text=True).stdout
            self._tests = {}
            for path in out.split():
                text = Path(ROOT, path).read_text(errors="replace")
                for pid in set(re.findall(r"corpus/problems/([\w-]+)/tests\b", text)):
                    self._tests.setdefault(pid, []).append(path)
        return self._tests

    def hillclimb(self):
        if self._hill is None:
            path = os.path.join(ROOT, "research", "design-history", "spec-delta", "hillclimb.md")
            self._hill = open(path).read() if os.path.exists(path) else ""
        return self._hill


def changed_ids(base):
    """Packages with a file changed since `base` (`staged`: the index)."""
    cmd = ["git", "diff", "--name-only", *(["--cached"] if base == "staged" else [base]), "--", "corpus/problems"]
    files = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True, check=True).stdout.split()
    return sorted({f.split("/")[2] for f in files if f.count("/") >= 3})


def findings(pkg, modes, ctx):
    return [f for m in modes for f in CHECKS[m](pkg, ctx)]

OUT = os.path.join(ROOT, "target", "corpus-tests")
INLINE = 200


def stamp(pkg, ref):
    """What a package's generated cases depend on; `make` skips one whose stamp is unchanged."""
    h = hashlib.sha256()
    for f in (pkg.tests, pkg.gen, ref):
        if f:
            h.update(Path(f).read_bytes())
    return h.hexdigest()


def make_cases(pkg, out=OUT):
    """Write <out>/<id>/NN.in from tests.toml and NN.ans from the accepted C++
    (an interactive package's NN.in is the interactor's hidden data, with no answer)."""
    ref = None if pkg.interactor else pkg.reference()
    if not pkg.interactor and ref is None:
        raise RuntimeError("no accepted C++ to make answers")
    tests = os.path.join(out, pkg.id)
    key = os.path.join(tests, ".stamp")
    if os.path.exists(key) and open(key).read() == stamp(pkg, ref):
        return
    exe = binary(ref) if ref else None
    shutil.rmtree(tests, ignore_errors=True)
    os.makedirs(tests)
    for i, case in enumerate(pkg.cases(), 1):
        text = pkg.case_input(case)
        Path(tests, f"{i:02}.in").write_text(text)
        if exe:
            r = subprocess.run([exe], input=text, capture_output=True, text=True, timeout=TIMEOUT, preexec_fn=judge_stack)
            if r.returncode:
                raise RuntimeError(f"accepted C++ exits {r.returncode} on case {i}")
            Path(tests, f"{i:02}.ans").write_text(r.stdout)
    Path(key).write_text(stamp(pkg, ref))


def make(args):
    wanted = [p for p in problems(sets=args.set or None, ids=args.ids or None) if p.tests]
    failed = []
    for pkg in wanted:
        try:
            make_cases(pkg, args.out or OUT)
        except (RuntimeError, subprocess.SubprocessError, OSError) as e:
            failed.append(f"{pkg.id}: {e!s:.160}")
    for f in failed:
        print(f, file=sys.stderr)
    print(f"corpus: cases for {len(wanted) - len(failed)} of {len(wanted)} packages")
    return 1 if failed else 0


def seed_toml(pkg, inputs):
    """tests.toml for `inputs` (name, text): a tiny one inline, a larger one a
    gen.py case of the largest power-of-two size whose input is no longer, plus
    `--max` when gen.py has it."""
    lines = ["# The package's cases (problem-packages/R9): `input` inline, else `gen.py seed size` or `args`.", ""]
    for n, (_, text) in enumerate(inputs, 1):
        if len(text) <= INLINE or not pkg.gen or pkg.interactor:
            if len(text) > INLINE and not pkg.interactor:
                raise RuntimeError(f"a {len(text)}-byte case and no gen.py")
            lines += ["[[case]]", "input = " + json.dumps(text), ""]
            continue
        size = 4
        while size < 1 << 20:
            try:
                if len(pkg.case_input({"seed": n, "size": size * 2}, timeout=TIMEOUT)) > len(text):
                    break
            except subprocess.SubprocessError:
                break
            size *= 2
        lines += ["[[case]]", f"seed = {n}", f"size = {size}", ""]
    if pkg.gen:
        for n, size in enumerate(STRESS_SIZES, len(inputs) + 1):
            lines += ["[[case]]", f"seed = {n}", f"size = {size}", ""]
    if has_max(pkg) and not pkg.interactor:
        lines += ["[[case]]", 'args = ["--max"]', ""]
    return "\n".join(lines).rstrip() + "\n"


def committed_inputs(pkg):
    """The committed tests/ inputs: `*.in`, or an interactive package's `*.case`, samples first."""
    d = pkg.dir / "tests"
    files = [f for f in d.iterdir() if f.suffix in (".in", ".case")] if d.is_dir() else []
    order = lambda f: (not f.stem.startswith("sample"), int(re.sub(r"\D", "", f.stem) or 0), f.name)
    return [(f.name, f.read_text()) for f in sorted(files, key=order)]


def seed(args):
    """Replace each package's committed tests/ with a tests.toml (problem-packages/R9)."""
    failed = 0
    for pkg in problems(ids=args.ids or None):
        if pkg.tests or not (pkg.dir / "tests").is_dir():
            continue
        try:
            text = seed_toml(pkg, committed_inputs(pkg))
        except (RuntimeError, subprocess.SubprocessError) as e:
            print(f"{pkg.id}: {e}", file=sys.stderr)
            failed += 1
            continue
        (pkg.dir / "tests.toml").write_text(text)
        shutil.rmtree(pkg.dir / "tests")
        print(f"{pkg.id}: tests/ -> tests.toml")
    return 1 if failed else 0



def verify(args):
    import concurrent.futures
    cpl = args.cpl or os.path.join(ROOT, "target", "release", "cpl")
    opt = lambda name, default=None: getattr(args, name, default) or default
    ids = list(args.ids)
    if opt("changed"):
        ids = changed_ids(args.changed)
        if not ids:
            print("corpus: no package changed")
            return 0
    wanted = list(problems(sets=opt("set"), ids=ids or None))
    missing = set(ids) - {p.id for p in wanted}
    if missing and not opt("changed"):
        sys.exit(f"corpus: no package {', '.join(sorted(missing))}")
    only = opt("only") or ("dream" if opt("check") else None)
    modes = only.split(",") if only else [m for m in MODES if m not in (opt("skip") or "").split(",")]
    bad = [m for m in modes if m not in CHECKS]
    if bad:
        sys.exit(f"corpus: no mode {', '.join(bad)}; modes: {', '.join(MODES)}")
    ctx = Context(cpl, run=not opt("no_run"), check=bool(opt("check")), named=() if opt("changed") else ids)
    try:
        with concurrent.futures.ThreadPoolExecutor(opt("j", os.cpu_count() or 4)) as pool:
            found = sorted(f for fs in pool.map(lambda p: findings(p, modes, ctx), wanted) for f in fs)
    except InfraError as e:
        print(f"corpus: no verdict, the toolchain failed: {e}", file=sys.stderr)
        return 2
    for f in found:
        print(f)
    print(f"corpus: verified {len(wanted)} package(s) ({','.join(modes)}): {len(found)} finding(s)", file=sys.stderr)
    return 1 if found else 0


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


def gaps(args):
    """What the dreams need: `cpl check` errors on every dream.cp, filed by gap."""
    cpl = args.cpl or os.environ.get("CPL") or os.path.join(ROOT, "target", "release", "cpl")
    paths = [os.path.abspath(p) for p in args.paths] or [str(p.dream) for p in problems() if p.dream]
    r = report(cpl, paths)
    if args.json:
        json.dump(r, sys.stdout, indent=2)
        print()
        return 0
    print("\n".join(summary_lines(r, args.top)))
    unnamed = [(f["dream"], f["errors"][0]["message"]) for f in r["files"] if f["errors"] and f["errors"][0]["gap"] is None]
    if unnamed:
        print("first errors that name no gap:")
        for dream, message in unnamed:
            print("  %s: %s" % (dream, message))
    return 0


def cli(argv):
    ap = argparse.ArgumentParser(usage=__doc__)
    sub = ap.add_subparsers(dest="mode", required=True)
    sub.add_parser("build")
    sub.add_parser("check")
    v = sub.add_parser("verify")
    v.add_argument("ids", nargs="*", help="package ids (default: every package)")
    v.add_argument("--set", action="append", help="only the packages in corpus/sets/<SET>.txt")
    v.add_argument("--changed", metavar="BASE", help="only packages changed since BASE (a commit, or `staged`)")
    v.add_argument("--only", help="comma-separated modes: " + ",".join(MODES))
    v.add_argument("--skip", help="comma-separated modes to leave out")
    v.add_argument("--no-run", action="store_true", help="skip everything that compiles or runs")
    v.add_argument("--cpl")
    v.add_argument("--check", action="store_true")
    v.add_argument("-j", type=int, default=os.cpu_count() or 4)
    m = sub.add_parser("make")
    m.add_argument("ids", nargs="*")
    m.add_argument("--set", action="append")
    m.add_argument("--out")
    sd = sub.add_parser("seed")
    sd.add_argument("ids", nargs="*")
    ls = sub.add_parser("ls", help="print each package's dir, or its FILE, relative to the repo root")
    ls.add_argument("ids", nargs="*")
    ls.add_argument("--set", action="append")
    ls.add_argument("--file", help="a package file such as port.cp; packages without it are left out")
    g = sub.add_parser("gaps")
    g.add_argument("paths", nargs="*")
    g.add_argument("--json", action="store_true")
    g.add_argument("--cpl")
    g.add_argument("--top", type=int, default=1000)
    args = ap.parse_args(argv)
    if args.mode == "verify":
        return verify(args)
    if args.mode == "make":
        return make(args)
    if args.mode == "seed":
        return seed(args)
    if args.mode == "gaps":
        return gaps(args)
    if args.mode == "ls":
        return ls_mode(args)
    return main(args.mode)


def ls_mode(args):
    """The one corpus listing other languages' tools read: one path per line."""
    for pkg in problems(args.set, args.ids):
        path = pkg.dir / args.file if args.file else pkg.dir
        if path.exists():
            print(os.path.relpath(path, ROOT))
    return 0


def toolchain(argv):
    """Modes that build C++ need reference_cxx(): stop and name it when it's missing."""
    if not argv or argv[0] not in ("verify", "make", "seed"):
        return
    try:
        reference_cxx()
    except RuntimeError as e:
        sys.exit(f"corpus.py: {e}")


if __name__ == "__main__":
    toolchain(sys.argv[1:])
    sys.exit(cli(sys.argv[1:]))
