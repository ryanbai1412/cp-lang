#!/usr/bin/env python3
"""The corpus package: one corpus/problems/<id>/ layout, one walker.

  scripts/corpus.py build          regenerate the generated fields of every
                                   meta.json and corpus/index.json, keeping
                                   hand-written fields
  scripts/corpus.py check          fail when a meta.json is stale, isn't
                                   byte-identical to meta_text's canonical
                                   form, or names a missing file
  scripts/corpus.py delete <id>... remove packages whose statement can't be
                                   found: the dir, their submissions and
                                   info.json entries, and their lines in
                                   corpus/sets and silent_unknowns.txt
  scripts/corpus.py verify <id>... [--only MODES] [--cpl PATH]
                                   verify packages; writes nothing, so its
                                   `dream` mode fails unless the dream agrees
                                   and meta.json already records that for this
                                   dream.cp (problem-packages/R10)
  scripts/corpus.py record <id>... [--cpl PATH]
                                   run each dream.cp against its accepted C++
                                   and record an agreement as meta.json's
                                   `dream_stress`

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
# Under ~/.cache/cpl/cxx, which CI caches across runs; the key holds the source and compiler.
CXX_CACHE = os.path.join(os.path.expanduser("~"), ".cache", "cpl", "cxx", "corpus")
HAND = {"tags": [], "difficulty": None, "dream": "none", "spec": "none", "notes": ""}
RECORDED = ("dream_stress", "statement")
STATUSES = {"none", "draft", "reviewed", "gold"}
EXT = {"CPP": ".cpp", "CPP03": ".cpp", "CPP11": ".cpp", "CPP14": ".cpp", "CPP17": ".cpp",
       "CPP20": ".cpp", "JAVA": ".java", "PY3": ".py", "PYPY3": ".py", "PY2": ".py", "C": ".c",
       "TEXT": ".txt"}


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
    grader: Optional[Path] = None

    @classmethod
    def at(cls, d):
        d = Path(d)
        f = lambda name: d / name if (d / name).is_file() else None
        return cls(id=d.name, dir=d, meta=f("meta.json"), port=f("port.cp"), spec=f("spec.cp"),
                   dream=f("dream.cp"), orig=f("orig.cpp"), gen=f("gen.py"), checker=f("checker.py"),
                   interactor=f("interactor.py"), tests=f("tests.toml"),
                   grader=f("grader.cpp") or f("dream.tests/grader.cpp"))

    def read_meta(self):
        return json.loads(self.meta.read_text()) if self.meta else {}

    @property
    def solution(self):
        """What the gates run: port.cp, else a dream of `@grader` fns linked with the
        package's grader, which runs a two-phase task's phases in their own processes
        as the judge does, so that task needs no stdin twin."""
        if self.port:
            return self.port
        if self.dream and self.grader and GRADER.search(self.dream.read_text(errors="replace")):
            return self.dream
        return None

    def reference(self):
        """The accepted C++: orig.cpp, else meta.json's first AC .cpp submission."""
        if self.orig:
            return self.orig
        for s in self.read_meta().get("submissions", []):
            f = s.get("file") or ""
            if s.get("result") == "AC" and f.endswith(".cpp") and os.path.exists(os.path.join(ROOT, f)):
                return Path(ROOT, f)
        return None

    def judge(self):
        """The judge the package's cp.toml names (`[judge] judge = ...`), else `codeforces`: what
        `cpl test` and the real judge build the port for."""
        toml = self.dir / "cp.toml"
        named = tomllib.loads(toml.read_text()).get("judge", {}).get("judge") if toml.is_file() else None
        return named or "codeforces"

    def gen_input(self, argv, timeout=60):
        """What gen.py prints for `argv`: `seed size`, or `--max`. A tests.toml's cases are
        made by `cpl test --make` alone (make_cases)."""
        text = subprocess.run([sys.executable, str(self.gen), *map(str, argv)], capture_output=True, text=True,
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
    """`src` built with reference_cxx(), cached by content and compiler under CXX_CACHE."""
    os.makedirs(CXX_CACHE, exist_ok=True)
    cxx, version = reference_cxx()
    data = Path(src).read_bytes()
    key = hashlib.sha1(data + b"\0gcc " + version.encode()).hexdigest()[:16]
    exe = os.path.join(CXX_CACHE, key)
    if not os.path.exists(exe):
        # Built aside and renamed in: a parallel check must never run a half-written binary.
        tmp = f"{exe}.{os.getpid()}.{threading.get_ident()}"
        r = subprocess.run([*cxx, "-o", tmp, str(src)], capture_output=True, text=True)
        if r.returncode and "target specific option mismatch" in r.stderr:
            # `#pragma GCC target("avx2")` before the includes: the judges' CPUs have AVX2.
            r = subprocess.run([*cxx, "-mavx2", "-o", tmp, str(src)], capture_output=True, text=True)
        if r.returncode:
            raise RuntimeError("c++: " + r.stderr[:200])
        os.replace(tmp, exe)
    else:
        os.utime(exe)  # used: CI's cache trim keeps it
    return exe


def judge_stack():
    """The judges' stack, so a deep recursion in the accepted C++ runs as it did there."""
    import resource
    resource.setrlimit(resource.RLIMIT_STACK, (resource.RLIM_INFINITY, resource.RLIM_INFINITY))


STRESS_SIZES = (4, 8, 16, 32)


def stress_inputs(pkg, cpl=None):
    """The cases tests.toml makes, else gen.py at small sizes: what a solution must agree on."""
    if pkg.tests:
        make_cases(pkg, cpl=cpl)
        return [p.read_text() for p in generated(pkg, cpl)]
    if pkg.gen:
        return [pkg.gen_input([i, s]) for i, s in enumerate(STRESS_SIZES * 2, 1)]
    return []


GRADER = re.compile(r"^\s*@grader\b", re.M)


def judge_args(pkg, source):
    """How `cpl test` judges `source`: its interactor, else its checker, else the answers;
    a program of `@grader` fns is linked with the package's grader.cpp, as the judge runs it."""
    judge = ["--interactor", str(pkg.interactor)] if pkg.interactor else \
        ["--checker", str(pkg.checker)] if pkg.checker else []
    graded = pkg.grader and GRADER.search(Path(source).read_text(errors="replace"))
    return judge + (["--grader", str(pkg.grader)] if graded else [])


def package_limits(pkg):
    """The problem's limits, from its port's `// limits:` line, so a dream is judged as the port is."""
    if not pkg.solution:
        return []
    ms, mb = limits(pkg.solution)
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
                r = subprocess.run([exe], input=text, capture_output=True, text=True, timeout=60, cwd=cases,
                                   preexec_fn=judge_stack)
                if r.returncode:
                    raise InfraError(f"{pkg.id}: accepted C++ exits {r.returncode} on case {i}")
                Path(cases, f"{i:02}.ans").write_text(r.stdout)
        r = subprocess.run([cpl, "test", str(source), "--tests", cases, "--format", "json", *judge_args(pkg, source), *package_limits(pkg)],
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
        ok, n = judged(pkg, pkg.dream, stress_inputs(pkg, cpl), cpl)
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
    write_meta(pkg.meta, meta)


HEADER_FIELDS = ("Source", "Problem", "Solution")
HEADER_LINE = re.compile(r"//\s*(Source|Problem|Solution|Bounds):")
# Any `// key:` header line, which ends the one before; an indented line continues it.
HEADER_KEY = re.compile(r"// ?([A-Za-z][\w-]*):")
TIMEOUT = 20
SLOWER, FLOOR = 1.5, 0.1
# One timed run at a time: two at once share caches and memory bandwidth, which
# CPU time counts against whichever side happens to run beside a heavy neighbour.
TIMED = threading.Lock()
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


READ_LINE = re.compile(r"^\s*(?:let|var) (?:(.+?): (.+) = read\(\)|([^:=]+?) = read<(.+)>\(\))\s*$")
DOMAIN_LINE = re.compile(r"^(?:domain|type) (\w+) = (.+?)\s*$")
STRUCT_LINE = re.compile(r"^struct (\w+)(\(.*\)):?\s*$")
ENUM_LINE = re.compile(r"^enum (\w+):$")
ENUM_VARIANT = re.compile(r"^(?:@\w+(?:\([^)]*\))?\s+)*\w+(\(.*\))$")
READ_RANGE = re.compile(r"^(-?[\w.]+(?:\s*[-+*/^]\s*[\w.]+)*)\s*\.\.(=?)\s*(.+)$")
def split_top(s):
    out, depth, cur = [], 0, ""
    for ch in s:
        if ch in "([": depth += 1
        if ch in ")]": depth -= 1
        if ch == "," and depth == 0: out.append(cur.strip()); cur = ""
        else: cur += ch
    return out + [cur.strip()] if cur.strip() else out
def strip_index(t):
    while t.startswith("["):
        d = 0
        for i, ch in enumerate(t):
            d += ch == "["; d -= ch == "]"
            if d == 0: t = t[i+1:].strip(); break
    return t
def read_ranges(names, t, domains):
    """name -> (lo, hi) text for every ranged leaf of read type t."""
    t = strip_index(t.strip())
    while m := re.fullmatch(r"List<(.*)>", t):
        t = strip_index(m.group(1).strip())
    t = domains.get(t, t)
    if t.startswith("(") and t.endswith(")"):
        parts = split_top(t[1:-1]); out = {}
        if len(names) == len(parts):
            for n, p in zip(names, parts): out.update(read_ranges([n], p, domains))
        elif len(names) == 1:
            labelled = [p for p in parts if ":" in p and not p.split(":")[0].strip().count(" ")]
            for i, p in enumerate(parts):
                # An unnamed tuple's fields are positional: `name.0`, `name.1`, ...
                f, ft = p.split(":", 1) if labelled else (str(i), p)
                if labelled and p not in labelled:
                    continue
                out.update(read_ranges([f"{names[0]}.{f.strip()}"], ft, domains))
        return out
    t = domains.get(t, t)
    if s := re.fullmatch(r"Str<(.+)>", t):
        # A string's range is its length: `s: Str<1..=n>` is stated as `s.len: 1..=n`.
        return {f"{n}.len": r for n, r in read_ranges(names, s.group(1), domains).items()}
    m = READ_RANGE.match(t)
    if not m: return {}
    hi = m.group(3).strip()
    return {n: (m.group(1).strip(), hi if m.group(2) else f"{hi} - 1") for n in names}
READ_BINDER = re.compile(r"^\s*(?:(?:let|var) (.+?)(?::[^=]+)?=(?!=)|for (.+?) in )")
def read_of(line):
    """The type of the first `read<T>()` or `read_line().parse<T>()` anywhere in `line`, or None."""
    m = re.search(r"(?<![\w])(?:read|read_line\(\)\.parse)<", line)
    if not m:
        return None
    depth = 0
    for j in range(m.end() - 1, len(line)):
        depth += {"<": 1, ">": -1}.get(line[j], 0)
        if depth == 0:
            return line[m.end():j] if line.startswith("()", j + 1) else None
    return None


READ_ARG = re.compile(r"(?<!\w)(?<!\w\.)read\(([^()]*)\)")
READ_DOMAIN = re.compile(r"^\s*domain (\w+) = read\(")


def arg_ranges(line, domains):
    """name -> (lo, hi) of each `read(lo..=hi)` in `line`, a number read and checked in the
    range. It is named for the domain it sizes (`domain I = read(1..=n)` is I.count), the
    list it sizes (`let a: [0..read(1..=n)]T` is a.len) or the name it binds (`let n =
    read(1..=n)`); one bound to no name is `read(lo..=hi)`, which no header can state."""
    out = {}
    for m in READ_ARG.finditer(line):
        if not READ_RANGE.match(m.group(1).strip()):
            continue
        d, b = READ_DOMAIN.match(line), READ_BINDER.match(line)
        pat = (b.group(1) or b.group(2)).strip() if b else ""
        if d:
            name = d.group(1) + ".count"
        elif re.fullmatch(r"\w+", pat) and re.search(r"\[(?:0\s*\.\.\s*)?$", line[:m.start()]):
            name = pat + ".len"
        elif re.fullmatch(r"\w+", pat):
            name = pat
        else:
            name = f"read({m.group(1).strip()})"
        out.update(read_ranges([name], m.group(1), domains))
    return out


def port_domains(text):
    """Each `domain`/`type`/`struct` the port declares: name -> its type text. An enum is
    the struct of its variants' named fields, so `ops.u` names `Join(u: W, v: W)`'s u."""
    domains, enum = {}, None
    for line in text.split("\n"):
        code = line.split("//")[0].rstrip()
        if enum and code[:1] in (" ", "\t"):
            v = ENUM_VARIANT.match(code.strip())
            for f in split_top(v.group(1)[1:-1]) if v else []:
                if ":" in f:
                    fields.setdefault(f.partition(":")[0].strip(), f.strip())
            domains[enum] = "(" + ", ".join(fields.values()) + ")"
            continue
        e = ENUM_LINE.match(code.strip())
        enum, fields = (e.group(1), {}) if e else (None, None)
        m = DOMAIN_LINE.match(code.strip()) or STRUCT_LINE.match(code.strip())
        if m: domains[m.group(1)] = m.group(2)
    return domains
def logical_lines(text):
    """The code lines of `text` without comments, a line that leaves a bracket open
    joined with the ones that close it: `let t = Rooted(\n    read<T>(),\n)` is one line."""
    out, depth = [], 0
    for line in text.split("\n"):
        code = line.split("//")[0].rstrip()
        out.append(out.pop() + " " + code.strip() if depth > 0 else code)
        depth = max(0, depth + sum(code.count(c) for c in "([{") - sum(code.count(c) for c in ")]}"))
    return out
def port_ranges(text):
    """name (a field as name.field, a positional one as name.0) -> (lo, hi) text of each
    ranged read in the port: `let x: T = read()`, or a `read<T>()` bound by `let`/`for`
    anywhere in its statement, wrapped over lines or not (`let xs = read<[n]T>().sorted()`,
    `for a in read<[m]T>():`), and each `read(lo..=hi)` (arg_ranges)."""
    domains, out = port_domains(text), {}
    for line in logical_lines(text):
        out.update(arg_ranges(line, domains))
        m, b = READ_LINE.match(line), READ_BINDER.match(line)
        if m:
            pat, t = (m.group(1) or m.group(3)).strip(), m.group(2) or m.group(4)
        elif b and (t := read_of(line)) is not None:
            pat = (b.group(1) or b.group(2)).strip()
        else:
            continue
        names = split_top(pat[1:-1]) if pat.startswith("(") else [pat]
        out.update(read_ranges(names, t, domains))
    return out


def declared_bounds(text):
    """The header's `// Bounds:` in `cpl new --constraints` form, as name -> (lo, hi)
    text, or None without one; a part it can't read maps to None. A part may name the
    port's domains and give a whole read type: `s: [q](1..=3, Col)` states s.0 and s.1."""
    lines, on = [], False
    for line in text.split("\n"):
        if not line.startswith("//"):
            break
        m = HEADER_KEY.match(line)
        if m:
            on = m.group(1) == "Bounds"
            if on:
                lines.append(line.split(":", 1)[1])
        elif on and line.startswith("//   "):
            lines.append(line[2:])
    if not lines:
        return None
    out, domains = {}, port_domains(text)
    for part in " ".join(lines).split(";"):
        if not part.strip():
            continue
        names, _, t = part.partition(":")
        if not t:
            out[part.strip()] = None
            continue
        for n in split_top(names):
            got = read_ranges([n], t, domains)
            out.update(got or {n: None})
    return out


def bound_value(text):
    """A bound as a number, or None when it names an input (`n`, `n * m`); `/` divides as Int does."""
    t = text.replace("_", "").replace("^", "**").replace(" ", "")
    if not re.fullmatch(r"[-+*/().0-9e]+", t) or re.search(r"[a-df-z]", t):
        return None
    try:
        return eval(t.replace("/", "//"), {"__builtins__": {}})
    except (SyntaxError, ZeroDivisionError, TypeError):
        return None


def port_consts(text):
    """Each numeric `const NAME = value` the port declares: name -> its value text."""
    found = re.finditer(r"(?m)^const (\w+)(?::[^=]+)? = (.+?)\s*$", text)
    return {m.group(1): m.group(2) for m in found if bound_value(m.group(2)) is not None}


def bounds_findings(pid, text):
    """Each ranged read the header's Bounds doesn't state, and each stated bound
    the port reads narrower or differently: a valid judge input it would reject."""
    declared, read, consts = declared_bounds(text), port_ranges(text), port_consts(text)
    if declared is None:
        return [f"no-header-bounds {pid} -"] if read else []
    header = text.split("\n")[:next((i for i, l in enumerate(text.split("\n")) if not l.startswith("//")), None)]
    if sum(1 for l in header if re.match(r"//\s*Bounds:", l)) > 1:
        return [f"bounds-twice {pid} -"]
    found = [f"bounds-unparsed {pid} {n}" for n, r in declared.items() if r is None]
    # A bound on an expression of reads (`m * n: 1..=40`, `sum(n): 1..=2e5`) is one max-short checks.
    joint = lambda n: not re.fullmatch(r"[\w.]+", n) and set(re.findall(r"[A-Za-z_][\w.]*", n)) - {"sum"} <= set(read)
    found += [f"bounds-unread {pid} {n}" for n in declared if declared[n] and n not in read and not joint(n)]
    found += [f"bounds-undeclared {pid} {n}" for n in read if n not in declared]
    for n, want in declared.items():
        if not want or n not in read:
            continue
        got = read[n]
        nums = [bound_value(re.sub(r"\w+", lambda w: f"({consts.get(w[0], w[0])})", x)) for x in (*got, *want)]
        if None in nums:
            narrow = [" ".join(a.split()) for a in got] != [" ".join(b.split()) for b in want]
        else:
            narrow = nums[0] > nums[2] or nums[1] < nums[3]
        if narrow:
            found.append(f"bounds-narrow {pid} {n}: reads {got[0]}..={got[1]}, statement {want[0]}..={want[1]}")
    return found


TOP_READ = re.compile(r"^(?:let|var) (\w+|\([\w, ]+\))(?::.+?)? = read(?:<.+>)?\(.*\)$")


def size_reads(text):
    """The top-level scalar reads that size the input, name -> their line index: each one a
    domain's bound or a read's index names (`domain Piece = 1..=n`, `let a: [q]T = read()`)."""
    lines = logical_lines(text)
    reads = {}
    for i, line in enumerate(lines):
        m = TOP_READ.match(line)
        if m:
            for n in split_top(m.group(1).strip("()")):
                reads.setdefault(n.strip(), i)
    sizing = [l for l in lines if re.match(r"^domain \w+ = ", l)]
    sizing += ["".join(re.findall(r"\[[^\]]*\]", l)) for l in lines if TOP_READ.match(l)]
    return {n: i for n, i in reads.items() if any(re.search(rf"(?<![\w.])" + re.escape(n) + r"(?![\w(])", l) for l in sizing)}


def read_prefix(text, upto):
    """The port's top-level declarations and reads through line `upto`, the program that reads
    the sizes and nothing else: what `cpl run --format json` takes to name each value."""
    keep = re.compile(r"^(?:use |domain |type \w+ = |const |@)")
    lines = logical_lines(text)[:upto + 1]
    kept = {i for i, l in enumerate(lines) if keep.match(l) or TOP_READ.match(l)}
    # A top-level `let` a kept line names (`let top = 1e8` under `domain Half = 0..=top / 2`).
    lets = {i: m[1] for i, l in enumerate(lines) if (m := re.match(r"(?:let|var) (\w+)(?::[^=]+)? = (?!read)", l))
            and not (i + 1 < len(lines) and lines[i + 1][:1].isspace())}
    while new := {i for i, name in lets.items() if i not in kept and any(
            re.search(rf"(?<![\w.]){name}\b", lines[k]) for k in kept if k > i)}:
        kept |= new
    return "\n".join(lines[i] for i in sorted(kept)) + "\n"


def case_prefix(text):
    """The port through the top-level statement holding its last read: every read a test case
    makes, without the work after them."""
    lines = logical_lines(text)
    last = max(i for i, l in enumerate(lines) if re.search(r"(?<![\w.])read(?:<.+>)?\(", l))
    end = next((j for j in range(last + 1, len(lines)) if lines[j] and not lines[j][0].isspace()), len(lines))
    return "\n".join(lines[:end]) + "\n"


READ_CALL = re.compile(r"(?<![\w.])read(?:<.+>)?\(")


def read_slice(text):
    """The port cut to its reads: each read, the blocks around it, and the definitions they name
    (`let`, `var`, `fn`, transitively), so a case reads its whole input without its work."""
    lines = logical_lines(text)
    depth = [len(l) - len(l.lstrip()) if l.strip() else None for l in lines]

    def body(i):
        j = i + 1
        while j < len(lines) and (depth[j] is None or depth[j] > depth[i]):
            j += 1
        return range(i + 1, j)

    def around(i):
        """`i` and the headers it sits under, with the `if` and `elif` an `elif` or `else` needs."""
        out, d = {i}, depth[i]
        chain = d is not None and lines[i].lstrip().startswith(("elif", "else"))
        for j in range(i - 1, -1, -1):
            if d is None or depth[j] is None or depth[j] > d:
                continue
            if depth[j] < d or chain:
                out.add(j)
                d = depth[j]
                chain = lines[j].lstrip().startswith(("elif", "else"))

        return out

    keep = re.compile(r"^(?:use |domain |type \w+ = |const |@)")
    kept = {i for i, l in enumerate(lines) if keep.match(l)}
    for i, l in enumerate(lines):
        if READ_CALL.search(l):
            kept |= around(i)
    defs = {}
    for i, l in enumerate(lines):
        if m := re.match(r"\s*(?:let|var) (\w+)(?::[^=]+)? = ", l):
            if not any(depth[j] is not None for j in body(i)):
                defs.setdefault(m[1], []).append({i})
        elif m := re.match(r"fn (\w+)", l):
            defs.setdefault(m[1], []).append({i, *body(i)})
    def sees(i, k):
        """Whether line `k` is in the scope of a definition on line `i`."""
        return i < k and all(depth[j] is None or depth[j] >= depth[i] for j in range(i + 1, k + 1))

    while True:
        uses = {(w, k) for k in kept for w in re.findall(r"(?<![\w.])\w+", lines[k])}
        new = set().union(*(b for w, k in uses for b in defs.get(w, ()) if sees(min(b), k))) - kept
        if not new:
            break
        for i in new:
            kept |= around(i)
    for i in sorted(kept):
        if lines[i].rstrip().endswith(":") and not any(j in kept for j in body(i)):
            kept |= set(body(i))
    return "\n".join(lines[i] for i in sorted(kept)) + "\n"


def capped_memory():
    """A child's address space capped at 4 GB, so a run that outgrows it fails alone instead of
    taking its worker down."""
    import resource
    resource.setrlimit(resource.RLIMIT_AS, (4 << 30, 4 << 30))


def read_values(cpl, program, data, where=None):
    """name -> value text of each top-level value `program` has after reading `data`, None when
    it fails, or TIMED_OUT past INTERP_TIMEOUT."""
    with tempfile.TemporaryDirectory(dir=where) as d:
        src = Path(d, "sizes.cp")
        src.write_text(program)
        try:
            r = subprocess.run([cpl, "run", str(src), "--format", "json"], input=data, capture_output=True, text=True,
                               timeout=INTERP_TIMEOUT, preexec_fn=capped_memory)
            return {v["name"]: v["value"] for v in json.loads(r.stdout)["values"]} if r.returncode == 0 else None
        except subprocess.TimeoutExpired:
            return TIMED_OUT
        except (ValueError, KeyError):
            return None


def short_of(pid, text, cases):
    """`max-short` for each sizing read whose --max value is under the statement's bound in
    every case: the bound as a number, or one naming other reads (`m: 1..=n - 1`) taken at
    that case's values. A bound on an expression (`r * c`, or `sum(n)` over a multitest's
    cases) that --max reaches saturates every read it names."""
    cases = cases if isinstance(cases, list) else [cases]
    declared, consts, found = declared_bounds(text) or {}, port_consts(text), []
    nums = {k: bound_value(v) for k, v in consts.items()}
    totals = {f"sum({k})": sum(c.get(k, 0) for c in cases) for c0 in cases for k in c0}

    def at(t, vals):
        t = re.sub(r"\bsum\(\w+\)", lambda w: f"({totals.get(w[0], w[0])})", t)
        return bound_value(re.sub(r"(?<![\w.])[A-Za-z_]\w*", lambda w: f"({vals.get(w[0], w[0])})", t))
    full = set()
    for e, r in declared.items():
        if r and not re.fullmatch(r"[\w.]+", e):
            for c in cases:
                got, hi = at(e, {**nums, **c}), at(r[1], {**nums, **c})
                if None not in (got, hi) and got >= hi:
                    full |= set(re.findall(r"[A-Za-z_]\w*", e)) - {"sum"}
    for n in size_reads(text):
        want, seen = declared.get(n), [c for c in cases if n in c]
        if not want or not seen or n in full:
            continue
        his = [(c[n], at(want[1], {**nums, **c})) for c in seen]
        if all(h is not None and v < h for v, h in his):
            found.append(f"max-short {pid} {n}: --max reads {max(v for v, _ in his)}, the statement allows {want[1]}")
    return found


def case_values(cpl, text, data, names, where=None):
    """Each multitest case's values of `names` on `data`, from a trace of the whole port watching
    them; None when it fails, or TIMED_OUT past INTERP_TIMEOUT."""
    with tempfile.TemporaryDirectory(dir=where) as d:
        src = Path(d, "port.cp")
        src.write_text(text)
        try:
            r = subprocess.run([cpl, "run", str(src), "--trace", "--watch", ",".join(names), "--format", "json"],
                               input=data, capture_output=True, text=True, timeout=INTERP_TIMEOUT,
                               preexec_fn=capped_memory)
            steps = json.loads(r.stdout)["runs"]["fast"]["steps"] if r.returncode == 0 else None
        except subprocess.TimeoutExpired:
            return TIMED_OUT
        except (ValueError, KeyError):
            return None
    if steps is None:
        return None
    cases = {}
    for st in steps:
        cases.setdefault(st.get("stage"), {}).update({k: v for k, v in st["values"].items() if k in names})
    return [c for c in cases.values() if c] or None


def max_short(pkg, cpl, data):
    """The max-short findings of `data`, the package's --max input (problem-packages/R6). Each
    program reads the sizes with less of the port's work than the next: its declarations and
    reads, the port up to them, then the whole port. Each runs beside the port, under its
    cp.toml."""
    text = pkg.port.read_text(errors="replace")
    sizes = size_reads(text)
    if not sizes:
        return []
    where, lines = pkg.port.parent, logical_lines(text)
    if re.search(r"(?m)^@!?multitest", text):
        below = lines[next(i for i, l in enumerate(lines) if re.match(r"@!?multitest", l)):]
        nested = any(re.search(r"(?<![\w.])read(?:<.+>)?\(", l) and not TOP_READ.match(l) for l in below)
        last = max(i for i, l in enumerate(lines) if TOP_READ.match(l))
        # With one case, or every read top-level, the declarations and reads keep each case's
        # input aligned; otherwise each case runs through its last read.
        one = not re.search(r"(?m)^@!?multitest\(", text) and data.split(None, 1)[:1] == ["1"]
        programs = [read_prefix(text, last)] if one or not nested else []
        programs += [read_slice(text), case_prefix(text), text]
        count = None if re.search(r"(?m)^@!?multitest\(", text) else int(data.split(None, 1)[0])
        for program in programs:
            got = case_values(cpl, program, data, sorted(sizes), where)
            # A cut that misreads its input loses alignment, so it never ends on exactly T cases.
            if got is not None and got is not TIMED_OUT and (count is None or len(got) == count):
                return short_of(pkg.id, text, [{k: v for k, v in c.items() if isinstance(v, int)} for c in got])
        return [f"max-unread {pkg.id} {'timeout' if got is TIMED_OUT else '-'}"]
    upto = max(sizes.values())
    for program in (read_prefix(text, upto), "\n".join(lines[:upto + 1]) + "\n", text):
        got = read_values(cpl, program, data, where)
        if got is not None and got is not TIMED_OUT:
            ints = {k: int(v) for k, v in got.items() if re.fullmatch(r"-?\d+", v)}
            return short_of(pkg.id, text, ints)
    return [f"max-unread {pkg.id} {'timeout' if got is TIMED_OUT else '-'}"]


MAX_SHORT = "research/corpus-max-short.json"


def max_args(pkg):
    """The arguments of each of the package's --max inputs: `--max`, then each tests.toml case
    whose args start with `--max`, so a statement with either-or bounds (B = 0 and P <= 4e5,
    or P <= 3e4) saturates each branch in a case of its own."""
    tests = getattr(pkg, "tests", None)
    cases = tomllib.loads(tests.read_text()).get("case", []) if tests else []
    more = [tuple(map(str, c["args"])) for c in cases if list(c.get("args", []))[:1] == ["--max"]]
    return list(dict.fromkeys([("--max",), *more]))


def merged_short(found):
    """The max_short findings of a package's --max inputs, one list per input: each input it
    can't read, and each sizing read short in every input, at the most any of them reads."""
    unread = sorted({f for fs in found for f in fs if f.startswith("max-unread ")})
    short = [[f for f in fs if f.startswith("max-short ")] for fs in found]
    key = lambda f: f.split(":")[0]
    reads = lambda f: int(re.search(r"reads (-?\d+)", f)[1])
    keep = set.intersection(*({key(f) for f in fs} for fs in short)) if short else set()
    return unread + [max((f for fs in short for f in fs if key(f) == k), key=reads) for k in sorted(keep)]


def max_short_of(job):
    """One package's max_short findings, in a worker process; gen.py failing is max-unread."""
    pid, cpl = job
    pkg, found = next(problems(ids=[pid])), []
    for args in max_args(pkg):
        data = gen_out(pkg, args)
        if data is None:
            return [f"max-unread {pid} gen-fails"]
        found.append(max_short(pkg, cpl, data.decode()))
    return merged_short(found)


def max_short_mode(args):
    """Every v03 port's --max against its Bounds (max_short), -j at a time: a `cpl run` of
    each, too slow for the commit tier, so ci-slow.yml runs it. `--write` records them in
    research/corpus-max-short.json for the shrink-only corpus_max_short and corpus_max_unread
    counters; without it, a record that differs from what it measures fails."""
    import concurrent.futures
    cpl = args.cpl or default_cpl()
    pkgs = [p for p in problems(sets=["v03"]) if p.port and not p.interactor and has_max(p)]
    with concurrent.futures.ProcessPoolExecutor(args.j) as ex:
        found = sorted(f for fs in ex.map(max_short_of, [(p.id, cpl) for p in pkgs]) for f in fs)
    got = {k: [f for f in found if f.startswith(k.replace("_", "-") + " ")] for k in ("max_short", "max_unread")}
    for f in found:
        print(f)
    path = Path(ROOT, MAX_SHORT)
    if args.write:
        path.write_text(json.dumps(got, indent=2) + "\n")
        return 0
    had = json.loads(path.read_text()) if path.is_file() else {}
    if had != got:
        for k in got:
            for f in sorted(set(got[k]) ^ set(had.get(k, []))):
                print(f"corpus: {'new' if f in got[k] else 'gone'}: {f}", file=sys.stderr)
        print(f"corpus: {MAX_SHORT} is stale; fix a new finding, or record a gone one with `max-short --write`",
              file=sys.stderr)
        return 1
    print(f"corpus: {len(pkgs)} --max input(s): {len(got['max_short'])} max-short, {len(got['max_unread'])} max-unread")
    return 0


DREAM_OVER = "research/dream-over-32kb.json"


def dream_size_of(job):
    """One dream's C++ for its package's judge, in a worker process: (id, bytes), or (id, None)
    when it doesn't build."""
    pid, cpl = job
    pkg = next(problems(ids=[pid]))
    with tempfile.TemporaryDirectory() as tmp:
        cpp = os.path.join(tmp, f"{pid}.cpp")
        r = subprocess.run([cpl, "build", "--judge", pkg.judge(), str(pkg.dream), "-o", cpp], capture_output=True)
        return pid, None if r.returncode else os.path.getsize(cpp)


def dream_sizes_mode(args):
    """Every dream.cp's C++ against the 32 KB every solution should fit (goal 5), -j at a time;
    dreams.yml runs it. `--write` records the dreams over it in research/dream-over-32kb.json
    for the shrink-only dream_over_32kb counter; without it, a record that differs from what
    it measures fails, and so does a dream that doesn't build."""
    import concurrent.futures
    cpl = args.cpl or default_cpl()
    pkgs = [p for p in problems() if p.dream]
    with concurrent.futures.ProcessPoolExecutor(args.j) as ex:
        sizes = sorted(ex.map(dream_size_of, [(p.id, cpl) for p in pkgs]))
    unbuilt = [pid for pid, n in sizes if n is None]
    over = [pid for pid, n in sizes if n is not None and n > TARGET_BYTES]
    for pid, n in sizes:
        if n is None or n > TARGET_BYTES:
            print(f"dream-over-32kb {pid} {n}" if n else f"dream-unbuilt {pid} -")
    if unbuilt:
        print(f"corpus: {len(unbuilt)} dream(s) don't build: {' '.join(unbuilt)}", file=sys.stderr)
        return 1
    got = {"dream_over_32kb": over}
    path = Path(ROOT, DREAM_OVER)
    if args.write:
        path.write_text(json.dumps(got, indent=2) + "\n")
        return 0
    had = json.loads(path.read_text()) if path.is_file() else {}
    if had != got:
        for pid in sorted(set(over) ^ set(had.get("dream_over_32kb", []))):
            print(f"corpus: {'new' if pid in over else 'gone'}: {pid}", file=sys.stderr)
        print(f"corpus: {DREAM_OVER} is stale; cut a new one under 32 KB, or record a gone one with "
              "`dream-sizes --write`", file=sys.stderr)
        return 1
    print(f"corpus: {len(pkgs)} dream(s), {len(over)} over 32 KB")
    return 0


def check_header(pkg, ctx):
    if not pkg.port:
        return []
    have = header_fields(pkg.port.read_text(errors="replace"))
    missing = [f"no-header-{f.lower()} {pkg.id} -" for f in HEADER_FIELDS if f not in have]
    return missing + bounds_findings(pkg.id, pkg.port.read_text(errors="replace"))


def check_check(pkg, ctx):
    """`cpl check` reports no error in port.cp or dream.cp (a warning never fails)."""
    found = []
    for kind, f in (("port", pkg.port), ("dream", pkg.dream)):
        if not f:
            continue
        r = subprocess.run([ctx.cpl, "check", str(f)], capture_output=True, text=True)
        if not re.search(r"(?m)^error", r.stderr):
            continue
        # The log is the only evidence a CI runner keeps: show the file's errors.
        print(f"corpus: {ctx.cpl} check {os.path.relpath(f, ROOT)}", file=sys.stderr)
        print("\n".join(r.stderr.splitlines()[:40]), file=sys.stderr)
        found.append(f"{kind}-check {pkg.id} -")
    return found


def tokens(data):
    return data.split()


def run_out(exe, data):
    """`exe`'s stdout on `data`, or None if it fails. It runs in a scratch directory, so
    a file it writes (an accepted C++ that opens output.txt) never lands in the repo."""
    try:
        with tempfile.TemporaryDirectory() as scratch:
            r = subprocess.run(exe if isinstance(exe, list) else [exe], input=data, capture_output=True,
                               timeout=TIMEOUT, cwd=scratch, preexec_fn=judge_stack)
    except subprocess.TimeoutExpired:
        return None  # timeout fails: every caller reports a None
    return r.stdout if r.returncode == 0 else None


def accepted_cpps(pkg):
    """orig.cpp alone when the package has one: it is the reference the answers
    come from, and an AC submission may cover only the judge's narrower data.
    Otherwise every AC C++ submission."""
    if pkg.orig:
        return [pkg.orig]
    out = []
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


INLINE_INPUT_LIMIT = 64 * 1024


def check_package(pkg, ctx):
    """problem-packages R1-R4: answers from an accepted C++, a checker where
    answers vary, a generator for interactive hidden data, a logged dream."""
    found = []
    if pkg.interactor and not pkg.gen:
        found.append(f"interactor-no-gen {pkg.id} -")
    if pkg.dir.joinpath("tests").exists():
        found.append(f"committed-tests {pkg.id} -")
    # Every walk parses tests.toml: a sample over 64 KiB goes in a package file its case names.
    cases = tomllib.loads(pkg.tests.read_text()).get("case", []) if pkg.tests else []
    found += [f"inline-input-over-64k {pkg.id} case={i},use=input_file" for i, c in enumerate(cases, 1)
              if len(c.get("input", "").encode()) > INLINE_INPUT_LIMIT]
    for size in range(1, 6) if pkg.gen and ctx.run else ():
        try:
            pkg.gen_input([1, size])
        except (subprocess.SubprocessError, OSError):
            found.append(f"gen-fails {pkg.id} size={size}")
            break
    if pkg.dream and pkg.spec:
        code = lambda p: [l.split("//")[0].rstrip() for l in p.read_text(errors="replace").splitlines() if l.split("//")[0].strip()]
        if code(pkg.dream) != code(pkg.spec) and not (pkg.read_meta().get("notes") or "").strip() \
                and not re.search(rf"\b{re.escape(pkg.id)}\b", ctx.hillclimb()):
            found.append(f"dream-unlogged {pkg.id} -")
    found += [f"stale-tests-path {pkg.id} {path}" for path in ctx.tests_paths().get(pkg.id, ())]
    for inp in generated(pkg, ctx.cpl, make=ctx.run) if pkg.tests else ():
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
            if len(exes) > 1:
                try:
                    make_cases(pkg, cpl=ctx.cpl)
                except ReferenceFails as e:
                    return found + [f"reference-fails {pkg.id} {e.case}"]
            for i, inp in enumerate(generated(pkg, ctx.cpl), 1) if len(exes) > 1 else ():
                data = inp.read_bytes()
                outs = {e: run_out(e, data) for e in exes}
                if None in outs.values():
                    found.append(f"reference-fails {pkg.id} {i}")
                    break
                if len({tuple(tokens(o)) for o in outs.values()}) > 1:
                    # A reference whose output changes on a rerun (seeded from the clock) can't say.
                    outs = {e: o for e, o in outs.items() if tokens(run_out(e, data) or b"") == tokens(o)}
                if len({tuple(tokens(o)) for o in outs.values()}) > 1:
                    found.append(f"needs-checker {pkg.id} {i}")
                    break
    return found


def generated(pkg, cpl=None, make=True):
    """The package's cases in target/corpus-tests/<id>/*.in, made again first when an
    older tests.toml, gen.py or reference made them, so a stale case is never judged.
    Without `make` (`verify --no-run`, which compiles nothing), the cases already there:
    making them builds the reference, 55 s for ioi19p4, and timed the static gate out."""
    if pkg.tests and make:
        try:
            make_cases(pkg, cpl=cpl)
        except (RuntimeError, subprocess.SubprocessError, OSError) as e:
            raise InfraError(f"can't make {pkg.id}'s cases: {e}") from e
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
    """`cpl test port.cp` passes every case `make` writes from tests.toml, judged
    like the judge would: answers from the accepted C++, or the checker or
    interactor, at the package's limits (problem-packages/R8)."""
    if not pkg.tests or not ctx.runs(pkg):
        return []
    if not pkg.interactor and pkg.reference() is None:
        return [f"no-reference {pkg.id} -"]
    try:
        make_cases(pkg, cpl=ctx.cpl)
    except ReferenceFails as e:
        return [f"reference-fails {pkg.id} {e.case}"]
    except (RuntimeError, subprocess.SubprocessError) as e:
        raise InfraError(f"{pkg.id}: {e}") from e
    # Without a program, making the cases has run the reference on every one: that is the check.
    if not pkg.solution:
        return []
    cases = cpl_cases(ctx.cpl, pkg.solution, "--tests", os.path.join(OUT, pkg.id),
                      *judge_args(pkg, pkg.solution), *package_limits(pkg))
    if isinstance(cases, str):
        return [f"{cases} {pkg.id} -"]
    found = [f"cpl-test-fails {pkg.id} {c['name']}" for c in cases if c["verdict"] not in (None, "ok")]
    if any(c["verdict"] is None for c in cases):
        found.append(f"cpl-test-unjudged {pkg.id} -")
    return found


def native(pkg, ctx):
    """The port's C++ for its package's judge, built with g++ -O2 (cached): (exe, bytes), or (None, 0)."""
    if pkg.id in ctx.natives:
        return ctx.natives[pkg.id]
    with tempfile.TemporaryDirectory() as tmp:
        cpp = os.path.join(tmp, f"{pkg.id}.cpp")
        r = subprocess.run([ctx.cpl, "build", "--judge", pkg.judge(), str(pkg.port), "-o", cpp], capture_output=True)
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


TIMED_OUT = "timed-out"


def rejects(cmd, data):
    """Whether `cmd` refuses `data` as out of range, or TIMED_OUT when it can't say in TIMEOUT."""
    try:
        r = subprocess.run(cmd, input=data, capture_output=True, timeout=TIMEOUT)
    except subprocess.TimeoutExpired:
        return TIMED_OUT
    return r.returncode != 0 and bool(RANGE.search(r.stderr))


def gen_out(pkg, args):
    try:
        timeout = MAX_GEN_TIMEOUT if "--max" in args else TIMEOUT
        return pkg.gen_input(list(args), timeout=timeout).encode()
    except (subprocess.SubprocessError, OSError):
        return None


# How long the interpreter may take on one stress case: the largest v03
# cases take it 32 s (cco24p1 11.in, 33 MB) and 34 s (noi10p2 21.in).
INTERP_TIMEOUT = int(os.environ.get("CPL_INTERP_TIMEOUT", 60))


def run_result(cmd, data, timeout=TIMEOUT):
    """(exit code, stdout, stderr) of `cmd` on `data`, or TIMED_OUT past `timeout`."""
    try:
        r = subprocess.run(cmd, input=data, capture_output=True, timeout=timeout)
    except subprocess.TimeoutExpired:
        return TIMED_OUT
    return r.returncode, r.stdout, r.stderr


def compare(run, interp, cxx, data):
    """Why `cpl run` and the port's C++ disagree on `data`, None when they
    agree, or TIMED_OUT when the interpreter times out, which has no verdict
    and so is a finding. A C++ timeout is retried once."""
    a = run(interp, data, INTERP_TIMEOUT)
    if a == TIMED_OUT:
        return TIMED_OUT
    b = run(cxx, data)
    if b == TIMED_OUT:
        b = run(cxx, data)
    if b == TIMED_OUT:
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
    for inp in generated(pkg, ctx.cpl):
        why = compare(run_result, [ctx.cpl, "run", str(pkg.port)], [exe], inp.read_bytes())
        if why == TIMED_OUT:
            found.append(f"interp-timeout {pkg.id} {inp.name}")
        elif why:
            found.append(f"differ {pkg.id} {inp.name}: {why}")
    ref = pkg.reference()
    orig = built(ref) if ref else None
    for seed in GEN_SEEDS if pkg.gen else ():
        for size in GEN_SIZES:
            data = gen_out(pkg, (seed, size))
            if data is None:
                found.append(f"gen-fails {pkg.id} {seed}/{size}")
            elif why := rejects([ctx.cpl, "run", str(pkg.port)], data):
                found.append(f"{'interp-timeout' if why == TIMED_OUT else 'invalid-input'} {pkg.id} {seed}/{size}")
            elif orig:
                got, want = run_out(exe, data), run_out(orig, data)
                if got is None or want is None:
                    found.append(f"no-verdict {pkg.id} {seed}/{size}: "
                                 f"{'the port' if got is None else 'the reference'} fails or passes {TIMEOUT} s")
                elif not agrees(pkg, data, got, want):
                    found.append(f"disagrees {pkg.id} {seed}/{size}")
    return found


def has_max(pkg):
    return bool(pkg.gen) and "--max" in pkg.gen.read_text(errors="replace")


LIMITS = re.compile(r"^//\s*limits:\s*(\d+)\s*ms,\s*(\d+)\s*MB", re.M)
MAX_GEN_TIMEOUT = 60
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
    long us = (u.ru_utime.tv_sec + u.ru_stime.tv_sec) * 1000000L + u.ru_utime.tv_usec + u.ru_stime.tv_usec;
    if (f) { fprintf(f, "%ld %ld\n", (long)u.ru_maxrss, us); fclose(f); }
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
    """(exit code, stdout, CPU seconds, peak MB) of one run, or None on a timeout. CPU
    time, not wall time: a loaded box shares the cores, not the program's own work."""
    with tempfile.TemporaryFile() as inp, tempfile.TemporaryFile() as out, \
            tempfile.NamedTemporaryFile("r") as peak:
        inp.write(data)
        inp.seek(0)
        p = subprocess.Popen([launcher(), peak.name, os.path.abspath(exe)], stdin=inp, stdout=out,
                             stderr=subprocess.DEVNULL, start_new_session=True)
        try:
            code = p.wait(timeout)
        except subprocess.TimeoutExpired:
            os.killpg(p.pid, signal.SIGKILL)
            p.wait()
            return None
        out.seek(0)
        kb, us = (int(x) for x in (peak.read().split() or ["0", "0"]))
        return code, out.read(), us / 1e6, kb // (1024 if sys.platform == "darwin" else 1) / 1024


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
        else:
            os.utime(exe)  # used: CI's cache trim keeps it
        return exe
    finally:
        if os.path.exists(cpp):
            os.remove(cpp)


def at_max(pkg, cpl, exe, orig, data, later=None, label="--max"):
    """The findings of the max input (`label`, its gen.py args): it passes the checked C++, fits
    the limits, matches the accepted C++. Given a `later` list, the timed runs go on it instead,
    to run once every build is done."""
    pid, port = pkg.id, str(pkg.port)
    ms, mb = limits(port)
    found = []
    safe = checked(pid, port, cpl)
    if not safe:
        found.append(f"no-build {pid} -")
    else:
        r = measure(safe, data, max(TIMEOUT, CHECKED_SLOWER * ms / 1000))
        if r is None:
            found.append(f"checked-over-limit {pid} {label}")
        elif r[0] != 0:
            found.append(f"traps {pid} {label}")
    if not exe:
        return found
    if later is not None:
        os.makedirs(MAX_INPUTS, exist_ok=True)
        fd, path = tempfile.mkstemp(dir=MAX_INPUTS)
        with os.fdopen(fd, "wb") as f:
            f.write(data)
        later.append(lambda: timed_max(pkg, exe, orig, Path(path), ms, mb, label))
        return found
    return found + timed_max(pkg, exe, orig, data, ms, mb, label)


# A --max input waiting for its timed runs.
MAX_INPUTS = os.path.join(ROOT, "target", "corpus-max-inputs")


def timed_max(pkg, exe, orig, data, ms, mb, label="--max"):
    """The port's judge build against the accepted C++ on the max input (`data`, or a file of it,
    removed once read): the over-limit, traps, disagrees and slow findings."""
    if isinstance(data, Path):
        path, data = data, data.read_bytes()
        path.unlink()
    pid, found = pkg.id, []
    # The port and the accepted C++ take turns, each going first in turn, and
    # each side keeps its best CPU time: a burst of load hits both sides.
    runs, wants = [], []
    with TIMED:
        for i in range(3):
            for side in ((exe, orig) if i % 2 == 0 else (orig, exe)):
                if side == exe:
                    runs.append(measure(exe, data, 2 * ms / 1000 + 1))
                elif side:
                    wants.append(measure(orig, data, 4 * ms / 1000 + 1))
            if any(r is None or r[0] != 0 for r in runs):
                return found + [f"over-limit {pid} {label}" if any(r is None for r in runs) else f"traps {pid} {label}"]
    mine, theirs = min(r[2] for r in runs), None
    if orig:
        if all(w is not None and w[0] == 0 for w in wants):
            if not agrees(pkg, data, runs[0][1], wants[0][1]):
                found.append(f"disagrees {pid} {label}")
            theirs = min(w[2] for w in wants)
            if mine >= FLOOR and mine > SLOWER * max(theirs, FLOOR):
                found.append(f"slow {pid} {label}: {mine:.3f} s of CPU, {mine / max(theirs, FLOOR):.2f}x"
                             f" the accepted C++'s {theirs:.3f} s, over {SLOWER}x")
        else:
            found.append(f"reference-fails {pid} {label}")
    # On a box slower than the judge the accepted C++ itself runs past the
    # limit; the port then answers to the accepted C++'s time instead.
    if mine > max(ms / 1000, theirs or 0) or min(r[3] for r in runs) > mb:
        found.append(f"over-limit {pid} {label}")
    return found



def interacted(pkg, ctx):
    """The interactor rules the same on the interpreter and on the port's C++ for
    every case. An interactive case is no stdin to feed both sides: interactor.py
    talks to each, once per case, through `cpl test --interactor`."""
    cases = generated(pkg, ctx.cpl)
    if not cases:
        return []
    judge = ("--tests", str(cases[0].parent), "--interactor", str(pkg.interactor), *package_limits(pkg))
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
    """Each --max input (max_args) is valid and maximum: the port's checked C++ runs it
    without a trap, its judge build runs within the `// limits:` time and memory,
    and it matches the accepted C++ within 1.5x its time (problem-packages/R6).
    A missing --max is counted by the `corpus_no_max` ratchet, not here."""
    if not pkg.solution or not ctx.runs(pkg) or pkg.interactor or not has_max(pkg):
        return []
    inputs = []
    for args in max_args(pkg):
        data = gen_out(pkg, args)
        if data is None:
            return [f"gen-fails {pkg.id} {' '.join(args)}"]
        inputs.append((" ".join(args), data))
    if not pkg.port:
        # A dream through its grader: `cpl test` judges it at the limits.
        ok, _ = judged(pkg, pkg.solution, [d.decode() for _, d in inputs], ctx.cpl)
        return [] if ok else [f"max-fails {pkg.id} --max"]
    # A port the size check can't read yet is counted by corpus_max_unread, not failed here.
    found = [f for f in merged_short([max_short(pkg, ctx.cpl, d.decode()) for _, d in inputs])
             if f.startswith("max-short ")]
    exe, _ = native(pkg, ctx)
    ref = pkg.reference()
    orig = built(ref) if ref else None
    later = getattr(ctx, "later", None)
    for label, data in inputs:
        if exe and (why := rejects([exe], data)):
            found.append(f"{'max-timeout' if why == TIMED_OUT else 'invalid-input'} {pkg.id} {label}")
        elif ref and not orig:
            return found + [f"no-ref-build {pkg.id} {Path(ref).name}"]
        else:
            found += at_max(pkg, ctx.cpl, exe, orig, data, later, label)
    return found


def check_size(pkg, ctx):
    """The Codeforces build fits the 32 KB every solution should (goal 5)."""
    if not pkg.port or not ctx.runs(pkg):
        return []
    _, size = native(pkg, ctx)
    return [f"over-32kb {pkg.id} {size}"] if size > TARGET_BYTES else []


def check_lint(pkg, ctx):
    """No removed spelling `cpl migrate --check` would rewrite (idiomatic-code/R8), and
    every file already in `cpl fmt`'s canonical style."""
    files = [str(f) for f in (pkg.port, pkg.dream, pkg.spec) if f]
    if not files:
        return []
    found = []
    for tool, finding in (("migrate", "removed-spelling"), ("fmt", "fmt-dirty")):
        r = subprocess.run([ctx.cpl, tool, "--check", "--json", *files], capture_output=True, text=True)
        try:
            out = json.loads(r.stdout)["files"]
        except (ValueError, KeyError):
            found.append(f"{tool}-error {pkg.id} -")
            continue
        found += [f"{finding} {pkg.id} {Path(f['path']).name}" for f in out
                  if f["changed"] or f.get("diagnostics") or f.get("error")]
    return sorted(found)


def check_dream(pkg, ctx):
    """dream.cp agrees with the accepted C++ (problem-packages/R10), and meta.json records
    that for this dream.cp. It writes nothing: `corpus.py record <id>` writes the record."""
    if not (pkg.dream and pkg.meta and ctx.run):
        return []
    got = dream_stress(pkg, ctx.cpl)
    if not got["ok"]:
        return [f"dream-stress {pkg.id} fails-on-{got['cases']}-cases"]
    if (pkg.read_meta().get("dream_stress") or {}).get("sha") != got["sha"]:
        return [f"dream-unrecorded {pkg.id} run-scripts/corpus.py-record-{pkg.id}-and-stage-meta.json"]
    return []


def record_dreams(args):
    """Run each named package's dream.cp against its accepted C++ and record an agreement
    as meta.json's `dream_stress`, the record verify's `dream` mode checks. Every run
    finishes before anything is written, so a broken toolchain writes nothing."""
    import concurrent.futures
    cpl = args.cpl or default_cpl()
    require_cpl(cpl)
    wanted = [p for p in problems(sets=args.set, ids=args.ids or None) if p.dream and p.meta]
    if missing := set(args.ids) - {p.id for p in wanted}:
        sys.exit(f"corpus: no package with a dream.cp: {', '.join(sorted(missing))}")
    try:
        with concurrent.futures.ThreadPoolExecutor(args.j) as pool:
            got = list(pool.map(lambda p: (p, dream_stress(p, cpl)), wanted))
    except InfraError as e:
        print(f"corpus: no verdict, the toolchain failed: {e}", file=sys.stderr)
        return 2
    failed = []
    for pkg, verdict in got:
        if verdict["ok"]:
            record(pkg, "dream_stress", verdict)
        else:
            failed.append(f"dream-stress {pkg.id} fails-on-{verdict['cases']}-cases")
    for f in sorted(failed):
        print(f)
    print(f"corpus: recorded {len(got) - len(failed)} of {len(got)} dream(s)", file=sys.stderr)
    return 1 if failed else 0


CHECKS = {"header": check_header, "check": check_check, "package": check_package, "tests": check_tests,
          "stress": check_stress, "max": check_max, "size": check_size, "lint": check_lint, "dream": check_dream}


class Context:
    def __init__(self, cpl, run=True, named=()):
        self.named = set(named)
        self.cpl, self.run, self.natives, self._hill, self._tests = cpl, run, {}, None, None
        # The timed runs at_max leaves for after the builds; None runs them as it goes.
        self.later = None
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


class ReferenceFails(RuntimeError):
    """The accepted C++ crashes on a case tests.toml makes, or runs past its time budget:
    the case or the reference is wrong."""
    def __init__(self, case, why):
        super().__init__(f"accepted C++ {why} on case {case}")
        self.case = case


REFERENCE_FAILS = re.compile(r"accepted C\+\+ (exits .+?|runs past \d+ s) on case (\d+)")


def default_cpl():
    """The cpl corpus.py runs: $CPL, else the release build."""
    return os.environ.get("CPL") or os.path.join(ROOT, "target", "release", "cpl")


CPL_BUILD = "cargo build --release -p cpl-cli"


def cpl_sources(crate="cpl-cli", root=None):
    """The workspace files a `cpl` binary is built from: the workspace manifests and
    Cargo.lock, and the Cargo.toml, build.rs and src/ of cpl-cli and every crate it
    depends on by path."""
    root = root or ROOT
    files, seen, todo = [os.path.join(root, "Cargo.toml"), os.path.join(root, "Cargo.lock")], set(), [crate]
    while todo:
        name = todo.pop()
        manifest = os.path.join(root, "crates", name, "Cargo.toml")
        if name in seen or not os.path.isfile(manifest):
            continue
        seen.add(name)
        with open(manifest, "rb") as f:
            toml = tomllib.load(f)
        tables = [toml.get("dependencies", {}), toml.get("build-dependencies", {})]
        tables += [t.get("dependencies", {}) for t in toml.get("target", {}).values()]
        todo += [dep for t in tables for dep in t]
        files += [manifest, os.path.join(root, "crates", name, "build.rs")]
        for top, _, names in os.walk(os.path.join(root, "crates", name, "src")):
            files += [os.path.join(top, n) for n in names]
    return [f for f in files if os.path.isfile(f)]


def cpl_dep_info(cpl, root=None):
    """The files cargo built `cpl` from, by the `<cpl>.d` it writes beside it (std/
    through include_str!, build.rs inputs, every source), or None with no .d. It is
    the list cargo itself rebuilds by, so this agrees with cargo, and with CI's
    back-dated checkouts (scripts/ci-mtime.sh)."""
    try:
        with open(cpl + ".d", encoding="utf-8") as f:
            text = f.read().replace("\\\n", " ")
    except OSError:
        return None
    rule = next((l for l in text.splitlines() if re.match(r"(\\ |[^ ])+: ", l)), "")
    deps = re.split(r"(?<!\\)\s+", rule.partition(": ")[2].strip())
    base = root or ROOT
    paths = [os.path.join(base, d.replace("\\ ", " ")) for d in deps if d]
    # build.rs reruns on git's HEAD only to stamp `cpl --version`; every commit or
    # rebase touches it, and an older stamp runs the same compiler.
    return [f for f in paths if ".git" not in os.path.relpath(f, base).split(os.sep)]


def cpl_build(cpl):
    """The command that builds `cpl`: the debug build for target/debug, else the release one."""
    return "cargo build -p cpl-cli" if os.path.basename(os.path.dirname(cpl)) == "debug" else CPL_BUILD


def cpl_stale(cpl, root=None):
    """Why `cpl` can't be run, naming the command that builds it: it is missing, or it is a
    build under <root>/target older than a source it is built from (cargo's dep-info, else
    `cpl_sources`); None when it is current. corpus.py and debt-ratchet both check through it."""
    root = root or ROOT
    rel = os.path.relpath(cpl, root)
    if not os.path.exists(cpl):
        return f"no cpl at {rel}; build it with `{cpl_build(cpl)}`"
    target = os.path.join(root, "target")
    if os.path.commonpath([os.path.abspath(cpl), target]) == target:
        built = os.path.getmtime(cpl)
        inputs = cpl_dep_info(cpl, root)
        if inputs is None:
            inputs = cpl_sources(root=root)
        if any(not os.path.exists(f) or os.path.getmtime(f) > built for f in inputs):
            return f"{rel} is stale: run {cpl_build(cpl)}"
    return None


def current_cpl(root=None):
    """$CPL, else the newer current one of target/debug/cpl and target/release/cpl, as
    `cpl_stale` judges them: `(cpl, None)`, or `(cpl, why)` naming the build command when
    none is current. Every script that runs the built `cpl` picks it here."""
    root = Path(root or ROOT)
    if env := os.environ.get("CPL"):
        candidates = [Path(env)]
    else:
        built = [p for p in (root / "target/debug/cpl", root / "target/release/cpl") if p.is_file()]
        candidates = sorted(built, key=lambda p: p.stat().st_mtime, reverse=True) or [root / "target/debug/cpl"]
    for cpl in candidates:
        if not cpl_stale(str(cpl), str(root)):
            return cpl, None
    return candidates[0], cpl_stale(str(candidates[0]), str(root))


def require_cpl(cpl):
    """Exit with `cpl_stale`'s line when `cpl` is missing or stale: a stale binary may lack a
    command corpus.py runs (`cpl test --make`) and fail with its usage."""
    if why := cpl_stale(cpl):
        sys.exit(f"corpus: {why}")


def sharded(pkgs, shard):
    """Shard `N/M` of `pkgs`: every M-th by sorted id from the N-th, so the M shards
    cover each package once. CI's per-shard jobs split the corpus this way."""
    if not shard:
        return pkgs
    m = re.fullmatch(r"(\d+)/(\d+)", shard)
    n, of = (int(g) for g in m.groups()) if m else (0, 0)
    if not 1 <= n <= of:
        sys.exit(f"corpus: --shard {shard}: want N/M with 1 <= N <= M")
    return sorted(pkgs, key=lambda p: p.id)[n - 1::of]


def make_cases(pkg, out=OUT, cpl=None):
    """<out>/<id>/NN.in from tests.toml and NN.ans from the accepted C++ (an interactive
    package's NN.in is the interactor's hidden data, with no answer), and an empty NN.max
    beside each `args = ["--max"]` case: `cpl test --make`, the one expansion `cpl test` reads."""
    r = subprocess.run([cpl or default_cpl(), "test", "--make", str(pkg.dir), "--tests", os.path.join(out, pkg.id)],
                       cwd=ROOT, capture_output=True, text=True)
    if r.returncode:
        if m := REFERENCE_FAILS.search(r.stderr):
            raise ReferenceFails(m.group(2), m.group(1))
        lines = r.stderr.strip().splitlines()
        raise RuntimeError(lines[-1] if lines else f"cpl test --make exits {r.returncode}")
    made = Path(out, pkg.id)
    for f in made.iterdir() if made.is_dir() else ():
        os.utime(f)  # used, even when the stamp says current: CI's cache trim keeps it


def make_one(job):
    """Make one package's cases in a worker process: its id and error, or None."""
    pid, out, cpl = job
    try:
        make_cases(next(problems(ids=[pid])), out, cpl)
    except (RuntimeError, subprocess.SubprocessError, OSError) as e:
        return f"{pid}: {e!s:.160}"
    return None


def make(args):
    """Make the cases of every package with a tests.toml, -j at a time. Processes,
    each a `cpl test --make`."""
    import concurrent.futures, multiprocessing
    ids = list(args.ids)
    if getattr(args, "changed", None):
        ids = [p.id for p in problems(ids=changed_ids(args.changed))]
        if not ids:
            return 0
    wanted = sharded([p for p in problems(sets=args.set or None, ids=ids or None) if p.tests], getattr(args, "shard", None))
    if wanted:
        require_cpl(args.cpl or default_cpl())
    jobs = [(p.id, args.out or OUT, args.cpl) for p in wanted]
    with concurrent.futures.ProcessPoolExecutor(max(1, args.j), mp_context=multiprocessing.get_context("fork")) as pool:
        failed = [f for f in pool.map(make_one, jobs) if f]
    for f in failed:
        print(f, file=sys.stderr)
    print(f"corpus: cases for {len(wanted) - len(failed)} of {len(wanted)} packages")
    return 1 if failed else 0


def seed_toml(pkg, inputs):
    """tests.toml for `inputs` (name, text): a tiny one inline (over 64 KiB, as an
    input_file in samples/), a larger one a gen.py case of the largest power-of-two size whose input is no longer, plus
    `--max` when gen.py has it."""
    lines = ["# The package's cases (problem-packages/R9): `input` inline, else `gen.py seed size` or `args`.", ""]
    for n, (_, text) in enumerate(inputs, 1):
        if len(text) <= INLINE or not pkg.gen or pkg.interactor:
            if len(text) > INLINE and not pkg.interactor:
                raise RuntimeError(f"a {len(text)}-byte case and no gen.py")
            if len(text.encode()) > INLINE_INPUT_LIMIT:
                (pkg.dir / "samples").mkdir(exist_ok=True)
                (pkg.dir / "samples" / f"{n}.in").write_text(text)
                lines += ["[[case]]", f'input_file = "samples/{n}.in"', ""]
                continue
            lines += ["[[case]]", "input = " + json.dumps(text), ""]
            continue
        size = 4
        while size < 1 << 20:
            try:
                if len(pkg.gen_input([n, size * 2], timeout=TIMEOUT)) > len(text):
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
    """The committed tests/ inputs, samples first: `*.in`, but an interactive
    package's `*.case` where it has them, since its `*.in` is then the reply
    transcript the interactor writes, not data the interactor can read."""
    d = pkg.dir / "tests"
    files = [f for f in d.iterdir() if f.suffix in (".in", ".case")] if d.is_dir() else []
    if pkg.interactor and any(f.suffix == ".case" for f in files):
        files = [f for f in files if f.suffix == ".case"]
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



# A package past this is named as it finishes, so a CI job cut off by its
# timeout still says where its time went.
SLOW_PACKAGE = 30


def verify(args):
    import concurrent.futures
    cpl = args.cpl or default_cpl()
    opt = lambda name, default=None: getattr(args, name, default) or default
    ids = list(args.ids)
    if opt("changed"):
        ids = changed_ids(args.changed)
        if not ids:
            print("corpus: no package changed")
            return 0
    wanted = sharded([p for p in problems(sets=opt("set"), ids=ids or None) if not opt("dreams") or p.dream], opt("shard"))
    missing = set(ids) - {p.id for p in wanted}
    if missing and not opt("changed"):
        sys.exit(f"corpus: no package {', '.join(sorted(missing))}")
    only = opt("only")
    modes = only.split(",") if only else [m for m in MODES if m not in (opt("skip") or "").split(",")]
    bad = [m for m in modes if m not in CHECKS]
    if bad:
        sys.exit(f"corpus: no mode {', '.join(bad)}; modes: {', '.join(MODES)}")
    if not opt("no_run") and set(modes) - {"header"}:
        require_cpl(cpl)
    ctx = Context(cpl, run=not opt("no_run"), named=() if opt("changed") else ids)
    def timed(pkg):
        t0 = time.monotonic()
        fs = findings(pkg, modes, ctx)
        if (secs := time.monotonic() - t0) > SLOW_PACKAGE:
            print(f"corpus: {pkg.id} took {secs:.0f} s", file=sys.stderr, flush=True)
        return fs
    # The timed --max runs wait until every package is built: a g++ beside them slows
    # the port more than the accepted C++ (noi21p1: 1.25x alone, 1.61x beside builds).
    ctx.later = [] if "max" in modes else None
    try:
        with concurrent.futures.ThreadPoolExecutor(opt("j", os.cpu_count() or 4)) as pool:
            found = [f for fs in pool.map(timed, wanted) for f in fs]
        found = sorted(found + [f for job in ctx.later or () for f in job()])
    except InfraError as e:
        print(f"corpus: no verdict, the toolchain failed: {e}", file=sys.stderr)
        return 2
    if opt("base") and found:
        found = [f for f in found if f not in base_findings(found, modes, opt("base"), cpl)]
    for f in found:
        print(f)
    print(f"corpus: verified {len(wanted)} package(s) ({','.join(modes)}): {len(found)} finding(s)", file=sys.stderr)
    return 1 if found else 0


def base_findings(found, modes, base, cpl, run=subprocess.run):
    """The findings in `found` that the packages' files at `base` have too (main's red, its
    owner routed, not refused); none when the base can't give a verdict (yac6p6's 08.in
    timed out on main's own port.cp, and refused idioms' land)."""
    sys.path.insert(0, os.path.join(ROOT, "scripts"))
    import new_reds
    from culprit import describe
    rev = new_reds.git("rev-parse", base, cwd=ROOT)
    common = Path(new_reds.git("rev-parse", "--path-format=absolute", "--git-common-dir", cwd=ROOT))
    ids = sorted({f.split()[1] for f in found if len(f.split()) > 1})
    with new_reds.base_slot(common, rev) as (wt, target):
        # The base gets half the timeout: a case near it (btoi06p5 08.in, 56 s either side of 60) times
        # out or not with the box's load, so only a base well inside it makes a candidate timeout new.
        # The base target sits under the main checkout's .git, so its dev cpl's std guard would
        # see that checkout, not `wt`, and give no verdict (idioms' four interp timeouts).
        env = {**os.environ, "CARGO_TARGET_DIR": str(target), "CPL_INTERP_TIMEOUT": str(INTERP_TIMEOUT // 2),
               "CPL_STD_DIR": str(wt / "std")}
        print(f"corpus: rerunning {len(ids)} package(s) with findings at the base {rev[:9]}", file=sys.stderr)
        profile = "debug" if "debug" in Path(cpl).parts else "release"  # the timeouts are the candidate's build's
        build = ["cargo", "build", "-q", "-p", "cpl-cli", *(["--release"] if profile == "release" else [])]
        if (b := run(build, cwd=wt, env=env, capture_output=True, text=True)).returncode:
            # Every finding then counts as new; say why, so a refusal names a broken base, not the change.
            print(f"corpus: no verdict at the base {rev[:9]}: `{' '.join(build)}` exited {b.returncode} in {wt}:\n"
                  + "\n".join(((b.stderr or "") + (b.stdout or "")).splitlines()[-15:]), file=sys.stderr)
            return set()
        r = run([sys.executable, "scripts/corpus.py", "verify", *ids, "--only", ",".join(modes),
                 "--cpl", os.path.join(target, profile, "cpl")], cwd=wt, env=env, capture_output=True, text=True)
        if r.returncode not in (0, 1):
            print(f"corpus: no verdict at the base {rev[:9]}: its verify exited {r.returncode}:\n"
                  + "\n".join(((r.stderr or "") + (r.stdout or "")).splitlines()[-15:]), file=sys.stderr)
            return set()
        old = set(r.stdout.splitlines()) & set(found)
        with open(common / "cpl-preexisting-reds.log", "a") as log:
            for f in sorted(old):
                pkg = f.split()[1]
                sha = new_reds.git("log", "-1", "--format=%H", rev, "--", f"corpus/problems/{pkg}", cwd=ROOT, check=False)
                who = (describe(sha, ROOT)[1] if sha else "") or "unknown"
                print(f"corpus: pre-existing red, owner {who}: {f} at the base {rev[:9]} too; main's culprit job "
                      "routes it, not refusing", file=sys.stderr)
                log.write(f"{rev} {who} corpus:{f}\n")
        return old


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
    stored = defaultdict(list)
    for name in sorted(os.listdir(SUBS)):
        stem, ext = os.path.splitext(name)
        if ext != ".json":
            stored[stem].append(ext)
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
            f = next((sid + x for x in (EXT.get(e.get("language", ""), ""), *stored.get(sid, ()))
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
    return json.dumps(x, indent=1) + "\n"


def meta_text(meta):
    """meta.json's one canonical form, which every writer uses and `check` holds each file to:
    one-space indent, ASCII, each submission row on one line."""
    fields = []
    for k, v in meta.items():
        if k == "submissions" and v:
            body = "[\n" + ",\n".join("  " + json.dumps(r) for r in v) + "\n ]"
        else:
            body = json.dumps(v, indent=1).replace("\n", "\n ")
        fields.append(f" {json.dumps(k)}: {body}")
    return "{\n" + ",\n".join(fields) + "\n}\n"


def write_meta(path, meta):
    Path(path).write_text(meta_text(meta))


def index(metas):
    return {"problems": len(metas),
            "with_cpp_ac": sum(1 for m in metas.values() if m["cpp_ac"]),
            "dream": {s: sum(1 for m in metas.values() if m["dream"] == s) for s in sorted(STATUSES)},
            "spec": {s: sum(1 for m in metas.values() if m["spec"] == s) for s in sorted(STATUSES)},
            "by_source": dict(sorted(Counter(m["source"] for m in metas.values()).items()))}


def orphan_submissions(metas):
    """Submission files no package lists and no other tracked file names."""
    listed = {os.path.basename(r["file"]) for m in metas.values() for r in m["submissions"] if r["file"]}
    loose = [n for n in sorted(os.listdir(SUBS)) if not n.endswith(".json") and n not in listed]
    return [n for n in loose if subprocess.run(
        ["git", "grep", "-q", "-F", "corpus/submissions/" + n, "--", ".", ":!corpus/submissions"],
        cwd=ROOT).returncode != 0]


PATH_REF = re.compile(r"(?<![\w./-])corpus/(problems|submissions)/([\w-](?:[\w.-]*[\w-])?)(\*)?")


def dangling_references():
    """(file, line, path) for each `corpus/problems/<id>` or `corpus/submissions/<file>`
    a tracked file outside corpus/ names that doesn't exist; a trailing `*` names a prefix.
    A script's tests are skipped: their fixtures name packages in a temporary tree."""
    out = subprocess.run(["git", "grep", "-n", "-I", "-E", "corpus/(problems|submissions)/", "--", ".", ":!corpus",
                          ":!scripts/test_*.py"],
                         cwd=ROOT, capture_output=True, text=True).stdout
    found = []
    for hit in out.splitlines():
        f, n, text = hit.split(":", 2)
        for m in PATH_REF.finditer(text):
            kind, name, star = m.groups()
            d = PROBLEMS if kind == "problems" else SUBS
            ok = any(e.startswith(name) for e in os.listdir(d)) if star else os.path.exists(os.path.join(d, name))
            if not ok:
                found.append((f, int(n), f"corpus/{kind}/{name}{star or ''}"))
    return found


def refs():
    """`corpus.py refs`: each dangling corpus reference, the part of `check` regenerating
    can't fix, so the commit gate blocks on it while a stale index only warns."""
    found = dangling_references()
    for f, n, path in found:
        print(f"{f}:{n}: names {path}, which doesn't exist", file=sys.stderr)
    return 1 if found else 0


def main(mode):
    gen = generate()
    metas, errors = {}, []
    for problem, g in gen.items():
        path = os.path.join(PROBLEMS, problem, "meta.json")
        try:
            meta = merged(g, path)
        except json.JSONDecodeError as e:
            errors.append(f"{rel(path)}: not JSON ({e}); fix it by hand")
            continue
        metas[problem] = meta
        for k in ("dream", "spec"):
            if meta[k] not in STATUSES:
                errors.append(f"{rel(path)}: {k} must be one of {sorted(STATUSES)}")
        for f in [r["file"] for r in meta["submissions"] if r["file"]] + \
                 [f for v in meta["artifacts"].values() for f in v]:
            if not os.path.exists(os.path.join(ROOT, f)):
                errors.append(f"{rel(path)}: missing {f}")
        text = Path(path).read_text() if os.path.exists(path) else None
        if mode == "build":
            os.makedirs(os.path.dirname(path), exist_ok=True)
            write_meta(path, meta)
        elif text != meta_text(meta):
            what = "not in canonical form" if text and json.loads(text) == meta else "stale"
            errors.append(f"{rel(path)}: {what}; run scripts/corpus.py build")
    for pkg in problems():
        if pkg.id not in gen:
            errors.append(f"corpus/problems/{pkg.id}: no submissions or artifacts")
    errors += [f"corpus/submissions/{n}: no package lists it; delete it or add it to info.json"
               for n in orphan_submissions(metas)]
    errors += [f"{f}:{n}: names {path}, which doesn't exist" for f, n, path in dangling_references()]
    ipath = os.path.join(ROOT, "corpus", "index.json")
    if mode == "build":
        Path(ipath).write_text(dump(index(metas)))
    elif not os.path.exists(ipath) or Path(ipath).read_text() != dump(index(metas)):
        errors.append("corpus/index.json: stale; run scripts/corpus.py build")
    for e in errors:
        print(e, file=sys.stderr)
    print(f"corpus: {len(metas)} problems, {len(errors)} error(s)")
    return 1 if errors else 0


def gaps(args):
    """What the dreams need: `cpl check` errors on every dream.cp, filed by gap."""
    cpl = args.cpl or default_cpl()
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
    sub.add_parser("refs")
    d = sub.add_parser("delete")
    d.add_argument("ids", nargs="+")
    v = sub.add_parser("verify")
    v.add_argument("ids", nargs="*", help="package ids (default: every package)")
    v.add_argument("--set", action="append", help="only the packages in corpus/sets/<SET>.txt")
    v.add_argument("--changed", metavar="BASE", help="only packages changed since BASE (a commit, or `staged`)")
    v.add_argument("--base", metavar="REV", help="a finding the packages have at REV too is main's: reported, not failed")
    v.add_argument("--dreams", action="store_true", help="only packages with a dream.cp")
    v.add_argument("--only", help="comma-separated modes: " + ",".join(MODES))
    v.add_argument("--skip", help="comma-separated modes to leave out")
    v.add_argument("--no-run", action="store_true", help="skip everything that compiles or runs")
    v.add_argument("--cpl")
    v.add_argument("-j", type=int, default=os.cpu_count() or 4)
    v.add_argument("--shard", metavar="N/M", help="only shard N of M (every M-th package by id)")
    r = sub.add_parser("record")
    r.add_argument("ids", nargs="*", help="package ids (default: every package with a dream.cp)")
    r.add_argument("--set", action="append", help="only the packages in corpus/sets/<SET>.txt")
    r.add_argument("--cpl")
    r.add_argument("-j", type=int, default=os.cpu_count() or 4)
    m = sub.add_parser("make")
    m.add_argument("ids", nargs="*")
    m.add_argument("--set", action="append")
    m.add_argument("--changed", metavar="BASE", help="only packages changed since BASE (a commit, or `staged`)")
    m.add_argument("--out")
    m.add_argument("--cpl")
    m.add_argument("-j", type=int, default=os.cpu_count() or 4)
    m.add_argument("--shard", metavar="N/M", help="only shard N of M (every M-th package by id)")
    ms = sub.add_parser("max-short", help="each v03 --max input that doesn't reach its port's Bounds")
    ms.add_argument("--write", action="store_true", help="record the findings in " + MAX_SHORT)
    ms.add_argument("--cpl")
    ms.add_argument("-j", type=int, default=os.cpu_count() or 4)
    ds = sub.add_parser("dream-sizes", help="each dream.cp whose C++ is over 32 KB")
    ds.add_argument("--write", action="store_true", help="record the findings in " + DREAM_OVER)
    ds.add_argument("--cpl")
    ds.add_argument("-j", type=int, default=os.cpu_count() or 4)
    sd = sub.add_parser("seed")
    sd.add_argument("ids", nargs="*")
    ls = sub.add_parser("ls", help="print each package's dir, or its FILE, relative to the repo root")
    ls.add_argument("ids", nargs="*")
    ls.add_argument("--set", action="append")
    ls.add_argument("--file", action="append",
                    help="a package file such as port.cp, or a glob such as '*.cp'; repeatable; packages without one are left out")
    g = sub.add_parser("gaps")
    g.add_argument("paths", nargs="*")
    g.add_argument("--json", action="store_true")
    g.add_argument("--cpl")
    g.add_argument("--top", type=int, default=1000)
    args = ap.parse_args(argv)
    if args.mode == "refs":
        return refs()
    if args.mode == "verify":
        return verify(args)
    if args.mode == "record":
        return record_dreams(args)
    if args.mode == "make":
        return make(args)
    if args.mode == "seed":
        return seed(args)
    if args.mode == "max-short":
        return max_short_mode(args)
    if args.mode == "dream-sizes":
        return dream_sizes_mode(args)
    if args.mode == "gaps":
        return gaps(args)
    if args.mode == "ls":
        return ls_mode(args)
    if args.mode == "delete":
        return delete(args.ids)
    return main(args.mode)


UNKNOWNS = os.path.join(ROOT, "crates", "cpl-conform", "silent_unknowns.txt")


def named_by(name, pid):
    """Tracked files outside package `pid` that mention `name`."""
    return subprocess.run(["git", "grep", "-l", "-F", name, "--", ".", ":!corpus/problems/" + pid],
                          cwd=ROOT, capture_output=True, text=True).stdout.split()


def delete(ids):
    """Remove packages and everything that names them, then rebuild the index,
    so a later `build` can't recreate one from info.json. A submission file
    another tracked file still names stays, without its info.json entry, and a
    path to a removed package or submission another tracked file still names is
    reported and fails the deletion. A hill-climb problem is refused before
    anything changes, since dropping it would shrink that fixed set."""
    known = {p.id for p in problems()}
    ipath = os.path.join(SUBS, "info.json")
    info = json.load(open(ipath))
    hill = Path(ROOT, "corpus", "hillclimb.json")
    climbs = set(json.loads(hill.read_text())["problems"]) if hill.exists() else set()
    for pid in ids:
        if pid in climbs:
            sys.exit(f"corpus: {pid} is in corpus/hillclimb.json; add a replacement picked by its criteria "
                     f"(sha1 of the problem id within its contest) and remove {pid} there first")
    removed = set()
    for pid in ids:
        subs = [sid for sid, e in info.items() if canonical_id(e["problem"]) == pid]
        if pid not in known and not subs:
            sys.exit(f"corpus: no package {pid}")
        for sid in subs:
            del info[sid]
            for f in Path(SUBS).glob(sid + ".*"):
                users = named_by(f.name, pid)
                if users:
                    print(f"{pid}: kept {rel(str(f))}, named by {', '.join(users)}")
                else:
                    f.unlink()
                    removed.add(f.name)
        for listing in [*Path(SETS).glob("*.txt"), Path(UNKNOWNS)]:
            if listing.exists():
                lines = listing.read_text().splitlines(keepends=True)
                kept = [l for l in lines if l.strip() != pid and f"corpus/problems/{pid}/" not in l]
                if kept != lines:
                    listing.write_text("".join(kept))
        for d in (Path(PROBLEMS) / pid, Path(ROOT, "target", "corpus-tests", pid),
                  Path(ROOT, "tests", "v02-snapshot", pid)):
            shutil.rmtree(d, ignore_errors=True)
        print(f"{pid}: deleted with {len(subs)} submission(s)")
    with open(ipath, "w") as f:
        f.write(json.dumps(info, indent=4))
    left = [(f, n, path) for f, n, path in dangling_references()
            if path.split("/")[2].rstrip("*") in {*ids, *removed}]
    for f, n, path in left:
        print(f"{f}:{n}: still names deleted {path}; drop that row or path")
    return main("build") or int(bool(left))


def ls_mode(args):
    """The one corpus listing other languages' tools read: one path per line."""
    for pkg in problems(args.set, args.ids):
        paths = sorted({f for leaf in args.file for f in pkg.dir.glob(leaf) if f.is_file()}) if args.file else [pkg.dir]
        for path in paths:
            print(os.path.relpath(path, ROOT))
    return 0


def toolchain(argv):
    """Modes that build C++ need reference_cxx(): stop and name it when it's missing."""
    if not argv or argv[0] not in ("verify", "record", "make", "seed"):
        return
    try:
        reference_cxx()
    except RuntimeError as e:
        sys.exit(f"corpus.py: {e}")


if __name__ == "__main__":
    toolchain(sys.argv[1:])
    sys.exit(cli(sys.argv[1:]))
