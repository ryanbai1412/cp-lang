#!/usr/bin/env python3
"""Unified validation scoreboard for CP Lang ports and friction reports.

Measures every auto-discovered port under proto/ports/ (ecnerwala and any
concurrently pushed Rust-era port directories with sibling *_orig.cpp files):

  - Gate status (cpl check, fmt, lint, build <= 64 KB; in full mode also
    differential C++ -O2 and interpreter verification, optional ASan/UBSan)
  - Contestant-written tokens: port vs raw original and library-stripped original
  - Emitted C++17 size in bytes (warn > 48 KB, fail > 64 KB)
  - Runtime and peak RSS of emitted C++ (-O2) vs original on a max-size input
  - Interpreter (`cpl run`) time
  - Friction summary over research/frictions/*.md and scripts/friction_lint.py
  - Diff against committed baseline research/scoreboard.json, failing on regressions

Usage:
    python3 scripts/scoreboard.py --fast --check          # cheap gate for check.sh (~3 s)
    python3 scripts/scoreboard.py --full --write          # full benchmark & update baseline
    python3 scripts/scoreboard.py --ports agc079a,cf2207e2
    python3 scripts/scoreboard.py --frictions             # friction report summary only
"""
import argparse
import concurrent.futures
import difflib
import glob
import hashlib
import json
import math
import os
import platform
import re
import resource
import shutil
import signal
import statistics
import subprocess
import sys
import tempfile
import threading
import time
from collections import Counter

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PORTS_DIR = os.path.join(ROOT, "proto", "ports")
FRICTIONS_DIR = os.path.join(ROOT, "research", "frictions")
REVIEWS_DIR = os.path.join(ROOT, "research", "reviews")
BUG_REPLAY_DIR = os.path.join(ROOT, "bench", "goal_tests", "bug_replay")
REVISION_DIR = os.path.join(ROOT, "bench", "goal_tests", "revision")
DEFAULT_BASELINE = os.path.join(ROOT, "research", "scoreboard.json")

SIZE_WARN = 48 * 1024
SIZE_LIMIT = 64 * 1024
PERF_FLOOR_MS = 10.0

# Regression thresholds for --check against baseline
EMITTED_GROWTH_RATIO = 1.05
EMITTED_GROWTH_BYTES = 1024
TOKEN_RATIO_TOLERANCE = 0.02
PERF_REGRESSION_RATIO = 1.10
TIME_P50_GOAL = 1.1
TIME_MAX_GOAL = 1.5
INTERP_RATIO_BAR = 10.0
INTERP_RATIO_NORTH_STAR = 5.0
INTERP_FLOOR_MS = 20.0
BUG_REPLAY_BAR_PCT = 90.0
GATE_FAST_BUDGET_S = 180.0
GATE_FAST_TIME_PATH = os.path.join(
    os.environ.get("CARGO_TARGET_DIR") or os.path.join(ROOT, "target"),
    "gate-fast-time.json",
)

# Adaptive max-size probe schedule for gen/NAME.py SEED SIZE
MAX_SIZE_STEPS = (8, 40, 200, 1000, 5000, 25000)
MAX_PROBE_ORIG_S = 0.25
MAX_PROBE_GEN_S = 2.0
MAX_PROBE_INPUT_BYTES = 20 * 1024 * 1024

RETIRED_DIR_PREFIXES = ("scout_batch_", "blind_critic_", "hillclimb_")
RETIRED_DIRS = {"sim3", "before", "wave1", "wave2"}

VALID_SEVERITIES = ("blocker", "high", "medium", "low")
VALID_GOALS = {
    "express",
    "hard-to-get-wrong",
    "performance",
    "evidence",
    "submission",
    "accumulate",
    "coherence",
}
_GOAL_ALIASES = {
    "express": "express",
    "express and revise": "express",
    "expressiveness": "express",
    "guessable apis": "express",
    "hard-to-get-wrong": "hard-to-get-wrong",
    "hard to get wrong": "hard-to-get-wrong",
    "safety": "hard-to-get-wrong",
    "performance": "performance",
    "performance and control": "performance",
    "evidence": "evidence",
    "doubt-to-evidence": "evidence",
    "doubt to evidence": "evidence",
    "submission": "submission",
    "dependable-submission": "submission",
    "dependable submission": "submission",
    "accumulate": "accumulate",
    "coherence": "coherence",
}
STDLIB_AREAS = (
    "graphs",
    "trees",
    "ranges",
    "strings",
    "math",
    "flows",
    "containers",
    "geometry",
    "dp",
    "seq",
)
_AREA_KEYWORDS = (
    ("flows", ("flow", "max_flow", "min_cost", "dinic", "matching", "bipartite", "hopcroft", "hungarian", "gomory")),
    ("geometry", ("geometry", "convex hull", "polygon", "cross product", "polar", "half-plane", "segment intersect", "point in polygon")),
    ("ranges", ("segtree", "lazysegtree", "fenwick", "sparsetable", "sparse table", "intervalmap", "interval partition", "odt", "range query", "segment tree", "mo's")),
    ("trees", ("hld", "lca", "centroid", "reroot", "euler tour", "permutation tree", "virtual tree", "treap", "link-cut", "tree")),
    ("strings", ("kmp", "z-func", "suffix", "aho", "rolling hash", "string", "`str`", "str ", "s[", "palindrom", "trie")),
    ("graphs", ("dijkstra", "bfs", "dfs", "scc", "2-sat", "twosat", "mst", "kruskal", "dsu", "toposort", "graph", "shortest path", "bridge", "articulation")),
    ("math", ("mod<", "comb<", "ntt", "fft", "convolv", "poly", "matrix", "gauss", "prime", "factor", "gcd", "crt", "binomial", "mint", "xor basis", "linear basis")),
    ("containers", ("sortedbag", "sortedset", "sortedmap", "minheap", "maxheap", "heap", "deque", "bitset", "map<", "set<", "counter", "unordered_map", "priority_queue", "hash map", "multiset")),
    ("dp", ("dp", "knapsack", "aliens", "monge", "cht", "li chao", "sos", "bitmask dp", "memo")),
    ("seq", ("std.seq", "runs", "chunk_by", "rotate", "reverse", "sorted(", "sort(", "dedup", "slice", "step(", "count(", "prod(", "sum(", "windows", "zip", "enumerate", "bsearch", "lower_bound")),
)

# Language-aware tokenizers (comments and whitespace stripped)
_CP_TOKEN_RE = re.compile(
    r"""
      //[^\n]*                                      # line comment (skip)
    | /\*[\s\S]*?\*/                                # block comment (skip)
    | "(?:\\.|[^"\\])*"                             # string literal
    | '(?:\\.|[^'\\])*'                             # char literal
    | 0[xXbBoO][0-9a-fA-F_]+                        # radix int literal
    | \d[\d_]*(?:\.\d[\d_]*)?(?:[eE][+-]?\d[\d_]*)? # int/float literal
    | [A-Za-z_]\w*                                  # identifier / keyword
    | \.\.=|\?\?=|<<=|>>=|::|\.\.|=>|->|\?\?|==|!=|<=|>=|<<|>>|\+=|-=|\*=|/=|%=|&=|\|=|\^=
    | [^\s]                                         # single punctuation/operator
    """,
    re.VERBOSE,
)

_CPP_TOKEN_RE = re.compile(
    r"""
      //[^\n]*                                      # line comment (skip)
    | /\*[\s\S]*?\*/                                # block comment (skip)
    | R"([^(]*)\([\s\S]*?\)\1"                      # raw string literal
    | "(?:\\.|[^"\\])*"                             # string literal
    | '(?:\\.|[^'\\])*'                             # char literal
    | 0[xXbB][0-9a-fA-F']+                          # radix int literal
    | \d[\d']*(?:\.\d[\d']*)?(?:[eE][+-]?\d[\d']*)?[uUlLfF]*
    | [A-Za-z_]\w*                                  # identifier / keyword
    | <=>|<<=|>>=|::|->\*|->|\.\.\.|<<|>>|<=|>=|==|!=|&&|\|\||\+\+|--|\+=|-=|\*=|/=|%=|&=|\|=|\^=
    | [^\s]                                         # single punctuation/operator
    """,
    re.VERBOSE,
)


def tokenize_cp(src):
    """Return list of lexical tokens in CP Lang source (no comments/whitespace)."""
    tokens = []
    for m in _CP_TOKEN_RE.finditer(src):
        tok = m.group(0)
        if tok.startswith("//") or tok.startswith("/*"):
            continue
        tokens.append(tok)
    return tokens


def tokenize_cpp(src):
    """Return list of lexical tokens in C++ source (no comments/whitespace)."""
    tokens = []
    for m in _CPP_TOKEN_RE.finditer(src):
        tok = m.group(0)
        if tok.startswith("//") or tok.startswith("/*"):
            continue
        tokens.append(tok)
    return tokens


def count_loc(src, block_comments=True):
    """Count non-blank, non-comment lines."""
    n = 0
    inside = False
    for raw in src.splitlines():
        s = raw.strip()
        if inside:
            if "*/" in s:
                inside = False
            continue
        if not s or s.startswith("//"):
            continue
        if block_comments and s.startswith("/*"):
            if "*/" not in s:
                inside = True
            continue
        n += 1
    return n


_CORPUS_REGISTRY = None


def _get_corpus_registry():
    global _CORPUS_REGISTRY
    if _CORPUS_REGISTRY is None:
        bench_dir = os.path.join(ROOT, "bench")
        if bench_dir not in sys.path:
            sys.path.insert(0, bench_dir)
        try:
            import corpus_registry as cr

            _CORPUS_REGISTRY = cr
        except Exception:
            _CORPUS_REGISTRY = False
    return _CORPUS_REGISTRY or None


def _read_text(path):
    with open(path, encoding="utf-8", errors="replace") as f:
        return f.read()


def strip_cpp_library(orig_path):
    """Return library-stripped C++ source using sibling .strip or bench/corpus_registry.auto_strip."""
    lines = _read_text(orig_path).splitlines()
    strip_file = re.sub(r"_orig\.cpp$", ".strip", orig_path)
    gone = set()
    if os.path.isfile(strip_file):
        for line in _read_text(strip_file).splitlines():
            s = line.strip()
            if s and not s.startswith("#"):
                try:
                    a, b = map(int, s.split()[0].split("-"))
                    gone |= set(range(a, b + 1))
                except ValueError:
                    pass
    else:
        cr = _get_corpus_registry()
        if cr is not None:
            gone = cr.auto_strip(lines)
    kept = [line for idx, line in enumerate(lines, 1) if idx not in gone]
    return "\n".join(kept)


def find_cpl_binary(override=None):
    """Locate the cpl binary (override > target/release/cpl > target/debug/cpl)."""
    if override:
        return override
    env_cpl = os.environ.get("CPL")
    if env_cpl and os.path.isfile(env_cpl):
        return env_cpl
    target_dir = os.environ.get("CARGO_TARGET_DIR") or os.path.join(ROOT, "target")
    for profile in ("release", "debug"):
        cand = os.path.join(target_dir, profile, "cpl")
        if os.path.isfile(cand):
            return cand
    return os.path.join(target_dir, "debug", "cpl")


def discover_ports(port_dirs=None, only=None):
    """Auto-discover Rust-era ports (*.cp with sibling *_orig.cpp) under proto/ports/."""
    dirs = []
    if port_dirs:
        for d in port_dirs:
            full = d if os.path.isabs(d) else os.path.join(ROOT, d)
            if os.path.isdir(full):
                dirs.append(full)
    else:
        if os.path.isdir(PORTS_DIR):
            for entry in sorted(os.listdir(PORTS_DIR)):
                full = os.path.join(PORTS_DIR, entry)
                if not os.path.isdir(full):
                    continue
                if entry in RETIRED_DIRS or entry.startswith(RETIRED_DIR_PREFIXES):
                    continue
                if glob.glob(os.path.join(full, "*_orig.cpp")):
                    dirs.append(full)

    ports = []
    for d in dirs:
        group = os.path.relpath(d, PORTS_DIR)
        for cp_path in sorted(glob.glob(os.path.join(d, "*.cp"))):
            stem = os.path.basename(cp_path)[:-3]
            orig_path = os.path.join(d, f"{stem}_orig.cpp")
            if not os.path.isfile(orig_path):
                continue
            port_id = f"{group}/{stem}"
            if only and port_id not in only and stem not in only:
                continue
            gen_path = os.path.join(d, "gen", f"{stem}.py")
            eps_path = os.path.join(d, "gen", f"{stem}.eps")
            interactor_path = os.path.join(d, "interactors", f"{stem}.py")
            checker_path = os.path.join(d, "checkers", f"{stem}.py")
            dream_path = os.path.join(d, "dream", f"{stem}.cp")
            sample_ext = "case" if os.path.isfile(interactor_path) else "in"
            samples = sorted(glob.glob(os.path.join(d, "samples", f"{stem}.*.{sample_ext}")))
            ports.append(
                {
                    "id": port_id,
                    "group": group,
                    "name": stem,
                    "path": os.path.relpath(cp_path, ROOT),
                    "orig_path": os.path.relpath(orig_path, ROOT),
                    "abs_path": cp_path,
                    "abs_orig": orig_path,
                    "gen": gen_path if os.path.isfile(gen_path) else None,
                    "eps": float(_read_text(eps_path).strip()) if os.path.isfile(eps_path) else None,
                    "interactor": interactor_path if os.path.isfile(interactor_path) else None,
                    "checker": checker_path if os.path.isfile(checker_path) else None,
                    "dream": dream_path if os.path.isfile(dream_path) else None,
                    "samples": samples,
                }
            )
    return ports


def file_sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        h.update(f.read())
    return h.hexdigest()[:16]


def sh(cmd, **kw):
    return subprocess.run(cmd, capture_output=True, text=True, **kw)


def big_stack():
    try:
        resource.setrlimit(resource.RLIMIT_STACK, (resource.RLIM_INFINITY, resource.RLIM_INFINITY))
    except (ValueError, OSError):
        pass


def run_timed(cmd, inp_path, out_path, timeout, rss=False):
    """Run cmd with stdin from inp_path; return (wall_s, peak_rss_kb, status)."""
    full_cmd = ["/usr/bin/time", "-f", "%M", "-o", out_path + ".rss", *cmd] if rss else list(cmd)
    with open(inp_path, "rb") as fi, open(out_path, "wb") as fo:
        t0 = time.perf_counter()
        pr = subprocess.Popen(
            full_cmd,
            stdin=fi,
            stdout=fo,
            stderr=subprocess.PIPE,
            preexec_fn=big_stack,
            start_new_session=True,
        )
        killer = threading.Timer(timeout, lambda: _kill_pg(pr.pid))
        killer.start()
        _, err = pr.communicate()
        wall = time.perf_counter() - t0
        timed_out = not killer.is_alive()
        killer.cancel()
    if timed_out:
        return timeout, 0, "timeout"
    kb = 0
    if rss:
        try:
            kb = int(_read_text(out_path + ".rss").split()[-1])
        except (OSError, ValueError, IndexError):
            pass
    if pr.returncode != 0:
        detail = (err.decode("utf-8", errors="replace").strip().splitlines() or [f"exit {pr.returncode}"])[0][:120]
        return wall, kb, f"exit {pr.returncode}: {detail}"
    return wall, kb, "ok"


def _kill_pg(pid):
    try:
        os.killpg(pid, signal.SIGKILL)
    except ProcessLookupError:
        pass


def run_interactive(interactor, case_path, binary_cmd, out_path, timeout):
    """Run an interactive port via its interactor script; return (wall_s, status)."""
    cmd = [sys.executable, interactor, case_path, "--", *binary_cmd]
    with open(out_path, "wb") as fo:
        t0 = time.perf_counter()
        pr = subprocess.Popen(
            cmd,
            stdout=fo,
            stderr=subprocess.PIPE,
            preexec_fn=big_stack,
            start_new_session=True,
        )
        killer = threading.Timer(timeout, lambda: _kill_pg(pr.pid))
        killer.start()
        _, err = pr.communicate()
        wall = time.perf_counter() - t0
        timed_out = not killer.is_alive()
        killer.cancel()
    if timed_out:
        return timeout, "timeout"
    if pr.returncode != 0:
        detail = (err.decode("utf-8", errors="replace").strip().splitlines() or [f"exit {pr.returncode}"])[0][:120]
        return wall, f"exit {pr.returncode}: {detail}"
    return wall, "ok"


def run_checker(checker_path, inp_path, out_path, ans_path, timeout=10.0):
    """Run checkers/NAME.py INPUT OUTPUT ANSWER. Returns (ok, err_summary)."""
    try:
        pr = subprocess.run(
            [sys.executable, checker_path, inp_path, out_path, ans_path],
            capture_output=True,
            text=True,
            timeout=timeout,
        )
        if pr.returncode == 0:
            return True, None
        msg = (pr.stderr.strip().splitlines() or pr.stdout.strip().splitlines() or [f"exit {pr.returncode}"])[0]
        return False, msg[:120]
    except subprocess.TimeoutExpired:
        return False, "checker timeout"


def judge_output(p, inp_path, ans_path, cand_path):
    """Judge candidate output against original answer using checker (if present) or token/eps compare."""
    if p.get("checker"):
        return run_checker(p["checker"], inp_path, cand_path, ans_path)
    return (outputs_match(ans_path, cand_path, p.get("eps")), None)


def outputs_match(path_a, path_b, eps=None):
    """Token-for-token comparison, with optional relative/absolute float tolerance."""
    a = _read_text(path_a).split()
    b = _read_text(path_b).split()
    if a == b:
        return True
    if len(a) != len(b):
        return False
    if eps is None:
        return False
    for x, y in zip(a, b):
        if x == y:
            continue
        try:
            fx, fy = float(x), float(y)
        except ValueError:
            return False
        if not (math.isfinite(fx) and math.isfinite(fy)):
            return False
        if abs(fx - fy) > eps * max(1.0, abs(fx), abs(fy)):
            return False
    return True


def compile_orig(orig_path, out_bin, work_dir):
    """Compile original C++ with g++ -std=c++20 -O2, falling back to zig c++ -std=c++23."""
    c = sh(["g++", "-std=c++20", "-O2", "-w", orig_path, "-o", out_bin])
    if c.returncode == 0:
        return True, "g++"
    if shutil.which("zig"):
        inc_dir = os.path.join(work_dir, "inc", "bits")
        os.makedirs(inc_dir, exist_ok=True)
        headers = (
            "algorithm array bitset cassert chrono climits cmath cstdint cstdio cstdlib "
            "cstring deque functional iomanip iostream map numeric optional queue random "
            "set sstream stack string tuple unordered_map unordered_set utility vector"
        ).split()
        with open(os.path.join(inc_dir, "stdc++.h"), "w") as f:
            for h in headers:
                f.write(f"#include <{h}>\n")
            f.write(
                "namespace std::ranges { template<class R, class T> "
                "void iota(R&& r, T v) { std::iota(std::begin(r), std::end(r), v); } }\n"
            )
        z = sh(["zig", "c++", "-std=c++23", "-O2", "-w", f"-I{os.path.join(work_dir, 'inc')}", orig_path, "-o", out_bin])
        if z.returncode == 0:
            return True, "zig c++"
        err = (z.stderr.strip().splitlines() or ["zig c++ failed"])[0][:140]
        return False, err
    err = (c.stderr.strip().splitlines() or ["g++ failed"])[0][:140]
    return False, err


def load_explicit_specs():
    """Load judge-limit specs from bench/corpus_generators.py and bench/rust_max_n_audit.py."""
    specs = {}
    bench_dir = os.path.join(ROOT, "bench")
    if bench_dir not in sys.path:
        sys.path.insert(0, bench_dir)
    try:
        import corpus_generators as cg

        for s in getattr(cg, "ECNERWALA_SPECS", []) + getattr(cg, "SPECS", []):
            specs[s[0]] = s
            specs.setdefault(os.path.basename(s[0]), s)
    except Exception:
        pass
    try:
        import rust_max_n_audit as rma

        for s in getattr(rma, "SPECS", []):
            specs[s[0]] = s
            specs[os.path.basename(s[0])] = s
    except Exception:
        pass
    return specs


def taste_counts(cpl, paths):
    """Per-file taste lint findings from `cpl lint --taste`: {path: (n, {code: n})}."""
    out = {}
    if not paths:
        return out
    r = sh([cpl, "lint", "--taste", *paths])
    for line in (r.stdout or "").splitlines():
        m = re.match(r"\s*(\d+)  (\S+)  (.*)$", line)
        if m and m.group(2) != "total":
            by = dict((c, int(n)) for c, n in (x.split(":") for x in m.group(3).split()))
            out[m.group(2)] = (int(m.group(1)), by)
    return out


def taste_by_lint(port_records):
    total = {}
    for r in port_records:
        for code, n in (r.get("taste_lints_by_lint") or {}).items():
            total[code] = total.get(code, 0) + n
    return dict(sorted(total.items()))


def run_batch_fast_checks(ports, cpl, jobs):
    """Run cpl check, fmt --check, lint, and parallel cpl build on all ports."""
    abs_paths = [p["abs_path"] for p in ports]
    rel_by_abs = {p["abs_path"]: p["path"] for p in ports}
    results = {
        p["id"]: {
            "check_ok": True,
            "check_err": None,
            "fmt_ok": True,
            "lint_warnings": 0,
            "lint_errors": 0,
            "taste_lints": 0,
            "taste_lints_by_lint": {},
            "build_ok": False,
            "emitted_bytes": None,
            "build_err": None,
        }
        for p in ports
    }
    id_by_path = {}
    for p in ports:
        id_by_path[p["path"]] = p["id"]
        id_by_path[p["abs_path"]] = p["id"]

    if not ports:
        return results

    # 1. cpl check (batch)
    chk = sh([cpl, "check", *abs_paths])
    if chk.returncode != 0:
        combined = (chk.stdout or "") + "\n" + (chk.stderr or "")
        for p in ports:
            # Look for `<path>: no errors` in stdout
            ok_marker = f"{p['abs_path']}: no errors"
            ok_rel = f"{p['path']}: no errors"
            if ok_marker not in combined and ok_rel not in combined:
                results[p["id"]]["check_ok"] = False
                for line in combined.splitlines():
                    if p["abs_path"] in line or p["path"] in line or p["name"] + ".cp" in line:
                        results[p["id"]]["check_err"] = line.strip()[:160]
                        break

    # 2. cpl fmt --check (batch)
    fmt = sh([cpl, "fmt", "--check", *abs_paths])
    if fmt.returncode != 0:
        combined = (fmt.stdout or "") + "\n" + (fmt.stderr or "")
        for line in combined.splitlines():
            m = re.match(r"would reformat\s+(.+)", line.strip())
            if m:
                fp = m.group(1).strip()
                pid = id_by_path.get(fp) or id_by_path.get(os.path.abspath(fp))
                if pid:
                    results[pid]["fmt_ok"] = False

    # 3. cpl lint (batch)
    lnt = sh([cpl, "lint", *abs_paths])
    combined_lint = (lnt.stdout or "") + "\n" + (lnt.stderr or "")
    if ", 0 warnings, 0 errors" not in combined_lint:
        for line in combined_lint.splitlines():
            for p in ports:
                if p["abs_path"] in line or p["path"] in line:
                    if "warning" in line.lower():
                        results[p["id"]]["lint_warnings"] += 1
                    elif "error" in line.lower():
                        results[p["id"]]["lint_errors"] += 1

    # 3b. taste lints (L05xx), counted apart: they are review findings, not gate failures
    for path, (n, by) in taste_counts(cpl, abs_paths).items():
        pid = id_by_path.get(path)
        if pid:
            results[pid]["taste_lints"] = n
            results[pid]["taste_lints_by_lint"] = by

    # 4. cpl build (parallel, measuring emitted bytes directly from stdout)
    def build_one(p):
        b = subprocess.run([cpl, "build", p["abs_path"], "-o", "-"], capture_output=True)
        if b.returncode == 0:
            return p["id"], True, len(b.stdout), None
        err_lines = [
            ln
            for ln in b.stderr.decode("utf-8", errors="replace").splitlines()
            if ln.strip() and "the header is left out" not in ln
        ]
        return p["id"], False, None, (err_lines[0][:160] if err_lines else f"exit {b.returncode}")

    with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as ex:
        for pid, ok, nbytes, err in ex.map(build_one, ports):
            results[pid]["build_ok"] = ok
            results[pid]["emitted_bytes"] = nbytes
            results[pid]["build_err"] = err

    return results


def measure_tokens(p):
    """Compute contestant-written token counts and LOC for a port and its original."""
    cp_src = _read_text(p["abs_path"])
    orig_src = _read_text(p["abs_orig"])
    stripped_src = strip_cpp_library(p["abs_orig"])

    cp_toks = len(tokenize_cp(cp_src))
    orig_toks = len(tokenize_cpp(orig_src))
    stripped_toks = len(tokenize_cpp(stripped_src))

    cp_loc = count_loc(cp_src, block_comments=True)
    orig_loc = count_loc(orig_src, block_comments=True)
    stripped_loc = count_loc(stripped_src, block_comments=True)

    dream_toks = None
    dream_ratio = None
    dream_gap = None
    if p.get("dream") and os.path.isfile(p["dream"]):
        dream_toks = len(tokenize_cp(_read_text(p["dream"])))
        dream_ratio = round(cp_toks / dream_toks, 4) if dream_toks else None
        dream_gap = cp_toks - dream_toks

    taste_score, review_rel = parse_review_score(p["name"])

    return {
        "cp_tokens": cp_toks,
        "orig_tokens": orig_toks,
        "orig_stripped_tokens": stripped_toks,
        "token_ratio": round(cp_toks / orig_toks, 4) if orig_toks else None,
        "stripped_token_ratio": round(cp_toks / stripped_toks, 4) if stripped_toks else None,
        "dream_tokens": dream_toks,
        "dream_ratio": dream_ratio,
        "dream_gap_tokens": dream_gap,
        "taste_score": taste_score,
        "review_path": review_rel,
        "cp_loc": cp_loc,
        "orig_loc": orig_loc,
        "orig_stripped_loc": stripped_loc,
        "cp_sha": file_sha256(p["abs_path"]),
        "orig_sha": file_sha256(p["abs_orig"]),
    }


_REVIEW_PENDING_RE = re.compile(
    r"(?im)^(?:[-*]\s*)?(?:#+\s*)?(?:\*\*)?(?:(?:port|taste)\s+)?score(?::)?(?:\*\*)?\s*(?:[:\-=—–]\s*)?(?:\*\*)?pending\b"
)
_REVIEW_SCORE_RE = re.compile(
    r"(?im)^(?:[-*]\s*)?(?:#+\s*)?(?:\*\*)?(?:(?:port|taste)\s+)?score(?::)?(?:\*\*)?\s*(?:[:\-=—–]\s*)?(?:\*\*)?([1-5](?:\.\d+)?)(?:\s*(?:/|out\s+of)\s*5)?(?:\*\*)?(?:\b|$)"
)
_REVIEW_SCORE_HEADING_RE = re.compile(
    r"(?im)^#+\s*(?:(?:port|taste)\s+)?score\s*\n+\s*(?:[-*]\s*)?(?:\*\*)?(pending|[1-5](?:\.\d+)?)(?:\s*(?:/|out\s+of)\s*5)?(?:\*\*)?(?:\b|$)"
)


def parse_review_score_text(text):
    """Parse a port taste score from review markdown: returns (score_or_None, is_pending)."""
    candidates = []
    m_pend = _REVIEW_PENDING_RE.search(text)
    if m_pend:
        candidates.append((m_pend.start(), "pending", None))
    m_num = _REVIEW_SCORE_RE.search(text)
    if m_num:
        candidates.append((m_num.start(), "score", m_num.group(1)))
    m_hdr = _REVIEW_SCORE_HEADING_RE.search(text)
    if m_hdr:
        raw = m_hdr.group(1)
        if raw.lower() == "pending":
            candidates.append((m_hdr.start(), "pending", None))
        else:
            candidates.append((m_hdr.start(), "score", raw))
    if not candidates:
        return None, False
    candidates.sort(key=lambda c: c[0])
    _, kind, raw = candidates[0]
    if kind == "pending":
        return None, True
    val = float(raw)
    if val.is_integer():
        val = int(val)
    return val, False


_DREAM_SCORE_RE = re.compile(r"(?m)^Dream:\s*(\d)\s*/\s*5\b")


def summarize_dream_scores(reviews_dir=REVIEWS_DIR):
    """Aggregate `Dream: N/5` scores across research/reviews/*.md."""
    if not os.path.isdir(reviews_dir):
        return {"scored": 0, "median": None, "under_4": 0, "min": None, "max": None}
    scores = []
    for fn in sorted(os.listdir(reviews_dir)):
        if not fn.endswith(".md"):
            continue
        m = _DREAM_SCORE_RE.search(_read_text(os.path.join(reviews_dir, fn)))
        if m:
            scores.append(int(m.group(1)))
    if not scores:
        return {"scored": 0, "median": None, "under_4": 0, "min": None, "max": None}
    med = statistics.median(scores)
    if isinstance(med, float) and med.is_integer():
        med = int(med)
    return {
        "scored": len(scores),
        "median": round(med, 2) if isinstance(med, float) else med,
        "under_4": sum(1 for s in scores if s < 4),
        "min": min(scores),
        "max": max(scores),
    }


def load_gate_fast_time(baseline=None, path=GATE_FAST_TIME_PATH):
    """Load the latest scripts/check.sh --fast wall time, falling back to baseline."""
    if path and os.path.isfile(path):
        try:
            with open(path, encoding="utf-8") as f:
                data = json.load(f)
            elapsed = data.get("elapsed_s")
            if elapsed is not None:
                elapsed_f = round(float(elapsed), 2)
                return {
                    "elapsed_s": elapsed_f,
                    "budget_s": GATE_FAST_BUDGET_S,
                    "ok": elapsed_f <= GATE_FAST_BUDGET_S,
                }
        except (OSError, ValueError, TypeError):
            pass
    if baseline:
        prev = (baseline.get("totals") or {}).get("gate_fast")
        if prev and prev.get("elapsed_s") is not None:
            elapsed_f = round(float(prev["elapsed_s"]), 2)
            return {
                "elapsed_s": elapsed_f,
                "budget_s": GATE_FAST_BUDGET_S,
                "ok": elapsed_f <= GATE_FAST_BUDGET_S,
            }
    return None


def parse_review_score(port_name, reviews_dir=REVIEWS_DIR):
    """Read taste score (1..5) from research/reviews/<port_name>.md if present."""
    path = os.path.join(reviews_dir, f"{port_name}.md")
    if not os.path.isfile(path):
        return None, None
    score, _ = parse_review_score_text(_read_text(path))
    return score, os.path.relpath(path, ROOT)


def _git_last_touch_order(repo_root, paths):
    """Return {rel_path: commit_index} where 0 is the most recent commit touching rel_path."""
    if not os.path.isdir(os.path.join(repo_root, ".git")):
        return {}
    try:
        r = subprocess.run(
            ["git", "log", "--format=format:COMMIT", "--name-only", "--", *paths],
            cwd=repo_root,
            capture_output=True,
            text=True,
            check=True,
        )
    except (OSError, subprocess.CalledProcessError):
        return {}
    first_seen = {}
    idx = -1
    for line in r.stdout.splitlines():
        if line == "COMMIT":
            idx += 1
        elif line and line not in first_seen:
            first_seen[line] = idx
    return first_seen


def summarize_taste_reviews(port_records, reviews_dir=REVIEWS_DIR, repo_root=ROOT):
    """Aggregate taste scores across all research/reviews/*.md plus stale/unreviewed counts."""
    scores = []
    pending = 0
    review_rel_by_name = {}
    if reviews_dir and os.path.isdir(reviews_dir):
        for fn in sorted(os.listdir(reviews_dir)):
            if not fn.endswith(".md") or fn == "README.md":
                continue
            name = os.path.splitext(fn)[0]
            full = os.path.join(reviews_dir, fn)
            rel = os.path.relpath(full, repo_root)
            review_rel_by_name[name] = rel
            sc, is_pend = parse_review_score_text(_read_text(full))
            if sc is not None:
                scores.append(sc)
            elif is_pend:
                pending += 1

    # Active ports without any review file
    active_cp_by_name = {}
    unreviewed = 0
    for rec in port_records:
        name = rec.get("name") or rec["id"].split("/")[-1]
        if rec.get("path"):
            active_cp_by_name[name] = rec["path"]
        if name not in review_rel_by_name:
            unreviewed += 1

    # Stale: ports whose .cp file changed in a commit newer than the last commit touching its review.
    stale = 0
    if review_rel_by_name:
        first_seen = _git_last_touch_order(repo_root, ["proto/ports", os.path.relpath(reviews_dir, repo_root)])
        if first_seen:
            extra_cp_by_name = {}
            ports_root = os.path.join(repo_root, "proto", "ports")
            if os.path.isdir(ports_root):
                for dirpath, dirnames, filenames in os.walk(ports_root):
                    dirnames[:] = [d for d in dirnames if d not in ("dream", "today")]
                    for fn in filenames:
                        if not fn.endswith(".cp"):
                            continue
                        pname = os.path.splitext(fn)[0]
                        if pname in active_cp_by_name or pname not in review_rel_by_name:
                            continue
                        rel = os.path.relpath(os.path.join(dirpath, fn), repo_root)
                        extra_cp_by_name.setdefault(pname, []).append(rel)

            for name, rrel in review_rel_by_name.items():
                rv_idx = first_seen.get(rrel)
                if rv_idx is None:
                    continue
                if name in active_cp_by_name:
                    cp_paths = [active_cp_by_name[name]]
                else:
                    cp_paths = extra_cp_by_name.get(name, [])
                if any(
                    (cp_idx := first_seen.get(cp)) is not None and cp_idx < rv_idx
                    for cp in cp_paths
                ):
                    stale += 1

    med = statistics.median(scores) if scores else None
    if isinstance(med, float) and med.is_integer():
        med = int(med)
    return {
        "reviewed": len(scores),
        "pending": pending,
        "median": round(med, 2) if isinstance(med, float) else med,
        "under_4": sum(1 for s in scores if s < 4),
        "min": min(scores) if scores else None,
        "max": max(scores) if scores else None,
        "histogram": {str(k): sum(1 for s in scores if int(round(s)) == k) for k in range(1, 6)},
        "stale": stale,
        "unreviewed": unreviewed,
    }


def generate_max_input(p, orig_bin, work_dir, explicit_specs):
    """Produce a max-size deterministic input for benchmarking port vs orig.

    Preference order:
      1. gen/NAME.py --max (when the generator supports --max)
      2. Explicit judge-limit spec in bench/corpus_generators.py / rust_max_n_audit.py
      3. Adaptive probe of gen/NAME.py 1 <SIZE> over MAX_SIZE_STEPS
      4. Largest sample in samples/
    Returns (input_path, input_source_label) or (None, reason).
    """
    if p["gen"] and not p["interactor"] and "--max" in _read_text(p["gen"]):
        inp = os.path.join(work_dir, "max_flag.in")
        try:
            with open(inp, "wb") as fo:
                pr = subprocess.run(
                    [sys.executable, p["gen"], "--max"],
                    stdout=fo,
                    stderr=subprocess.DEVNULL,
                    timeout=10.0,
                )
            if pr.returncode == 0 and 0 < os.path.getsize(inp) <= MAX_PROBE_INPUT_BYTES:
                return inp, "gen --max"
        except subprocess.TimeoutExpired:
            pass

    spec = explicit_specs.get(p["id"]) or explicit_specs.get(p["name"])
    if spec is not None:
        inp = os.path.join(work_dir, "max.in")
        try:
            def run_orig(path):
                run_timed([orig_bin], path, path + ".gen", 15.0, rss=False)
                return _read_text(path + ".gen")

            spec[4](inp, True, {"run_orig": run_orig})
            return inp, f"spec ({spec[3]})"
        except Exception:
            pass

    if p["gen"] and not p["interactor"]:
        best_path = None
        best_label = None
        prev_bytes = -1
        for size in MAX_SIZE_STEPS:
            cand = os.path.join(work_dir, f"probe_{size}.in")
            try:
                t0 = time.perf_counter()
                with open(cand, "wb") as fo:
                    pr = subprocess.run(
                        [sys.executable, p["gen"], "1", str(size)],
                        stdout=fo,
                        stderr=subprocess.DEVNULL,
                        timeout=MAX_PROBE_GEN_S,
                    )
                if pr.returncode != 0:
                    break
            except subprocess.TimeoutExpired:
                break
            nbytes = os.path.getsize(cand)
            if nbytes == 0 or nbytes > MAX_PROBE_INPUT_BYTES:
                break
            # Stop if generator caps output size internally
            if best_path is not None and nbytes == prev_bytes:
                break
            wall, _, st = run_timed([orig_bin], cand, cand + ".orig", 3.0, rss=False)
            if st != "ok":
                break
            best_path = cand
            best_label = f"gen size={size}"
            prev_bytes = nbytes
            if wall >= MAX_PROBE_ORIG_S:
                break
        if best_path:
            return best_path, best_label

    if p["samples"]:
        largest = max(p["samples"], key=lambda s: os.path.getsize(s))
        return largest, f"sample ({os.path.basename(largest)})"

    return None, "no generator or sample"


def evaluate_full_port(p, fast_res, cpl, explicit_specs, runs, cases, interp_timeout, run_san):
    """Run full differential gate, max-size C++ perf/RSS, and interpreter timing for one port."""
    out = {
        "gate_mode": "full",
        "perf": None,
        "interp": None,
        "diff_ok": False,
        "diff_detail": None,
    }
    if not (fast_res["check_ok"] and fast_res["build_ok"]):
        out["diff_detail"] = fast_res["check_err"] or fast_res["build_err"] or "fast check failed"
        return out

    with tempfile.TemporaryDirectory(prefix=f"cpl_sb_{p['name']}_") as work:
        cpp_path = os.path.join(work, "port.cpp")
        b = sh([cpl, "build", p["abs_path"], "-o", cpp_path])
        if b.returncode != 0:
            out["diff_detail"] = "cpl build failed"
            return out

        port_bin = os.path.join(work, "port")
        c = sh(["g++", "-std=c++17", "-O2", "-w", cpp_path, "-o", port_bin])
        if c.returncode != 0:
            err = (c.stderr.strip().splitlines() or ["g++ port failed"])[0][:140]
            out["diff_detail"] = f"g++ port: {err}"
            return out

        san_bin = None
        if run_san:
            san_bin = os.path.join(work, "port_san")
            cs = sh([
                "g++", "-std=c++17", "-O1", "-g",
                "-fsanitize=address,undefined", "-fno-sanitize-recover=all",
                cpp_path, "-o", san_bin,
            ])
            if cs.returncode != 0:
                err = (cs.stderr.strip().splitlines() or ["g++ san failed"])[0][:140]
                out["diff_detail"] = f"g++ san: {err}"
                return out

        orig_bin = os.path.join(work, "orig")
        ok_orig, orig_compiler = compile_orig(p["abs_orig"], orig_bin, work)
        if not ok_orig:
            out["diff_detail"] = f"orig compile: {orig_compiler}"
            return out

        # 1. Differential verification on samples + small generated cases (plus interpreter time)
        diff_cases = []
        for s in p["samples"]:
            diff_cases.append((s, f"sample {os.path.basename(s)}"))
        if p["gen"]:
            for seed in range(1, cases + 1):
                gin = os.path.join(work, f"gen_{seed}.in")
                try:
                    with open(gin, "wb") as fo:
                        pr = subprocess.run(
                            [sys.executable, p["gen"], str(seed), str(seed % 8 + 1)],
                            stdout=fo,
                            stderr=subprocess.DEVNULL,
                            timeout=5.0,
                        )
                    if pr.returncode == 0 and os.path.getsize(gin) > 0:
                        diff_cases.append((gin, f"gen seed={seed}"))
                except subprocess.TimeoutExpired:
                    pass

        interp_times = []
        interp_ok = True
        interp_err = None
        interp_src_label = None
        diff_ok = True
        diff_fail = None
        checked_count = 0

        for idx, (inp, label) in enumerate(diff_cases):
            o0 = os.path.join(work, f"o0_{idx}")
            o1 = os.path.join(work, f"o1_{idx}")
            o2 = os.path.join(work, f"o2_{idx}")
            if p["interactor"]:
                _, st0 = run_interactive(p["interactor"], inp, [orig_bin], o0, 10.0)
                if st0 != "ok":
                    diff_ok, diff_fail = False, f"orig {label}: {st0}"
                    break
                _, st1 = run_interactive(p["interactor"], inp, [san_bin or port_bin], o1, 15.0)
                if st1 != "ok":
                    diff_ok, diff_fail = False, f"port {label}: {st1}"
                    break
                if idx < 3:
                    iw, st2 = run_interactive(p["interactor"], inp, [cpl, "run", p["abs_path"]], o2, interp_timeout)
                    if st2 != "ok":
                        interp_ok, interp_err = False, f"{label}: {st2}"
                    else:
                        interp_times.append(iw * 1000.0)
                        if interp_src_label is None:
                            interp_src_label = label
            else:
                _, _, st0 = run_timed([orig_bin], inp, o0, 10.0, rss=False)
                if st0 != "ok":
                    diff_ok, diff_fail = False, f"orig {label}: {st0}"
                    break
                if p.get("checker"):
                    ok_chk0, err_chk0 = run_checker(p["checker"], inp, o0, o0)
                    if not ok_chk0:
                        diff_ok, diff_fail = False, f"checker rejected orig on {label}: {err_chk0}"
                        break
                _, _, st1 = run_timed([san_bin or port_bin], inp, o1, 15.0, rss=False)
                if st1 != "ok":
                    diff_ok, diff_fail = False, f"port {label}: {st1}"
                    break
                ok1, err1 = judge_output(p, inp, o0, o1)
                if not ok1:
                    diff_ok, diff_fail = False, f"cpp mismatch on {label}" + (f": {err1}" if err1 else "")
                    break
                if idx < 3:
                    iw, _, st2 = run_timed([cpl, "run", p["abs_path"]], inp, o2, interp_timeout, rss=False)
                    if st2 != "ok":
                        interp_ok, interp_err = False, f"{label}: {st2}"
                    else:
                        ok2, err2 = judge_output(p, inp, o0, o2)
                        if not ok2:
                            msg = f"interp mismatch on {label}" + (f": {err2}" if err2 else "")
                            interp_ok, interp_err = False, msg
                            diff_ok, diff_fail = False, msg
                            break
                        interp_times.append(iw * 1000.0)
                        if interp_src_label is None:
                            interp_src_label = label
            checked_count += 1

        out["diff_ok"] = diff_ok
        out["diff_detail"] = (
            f"{checked_count} cases ({orig_compiler})" if diff_ok else (diff_fail or "diff failed")
        )
        out["interp"] = {
            "ok": interp_ok,
            "ms": round(statistics.median(interp_times), 2) if interp_times else None,
            "cases": len(interp_times),
            "source": interp_src_label,
            "error": interp_err,
        }

        # 2. Max-size C++ (-O2) vs original runtime & peak RSS
        max_inp, max_label = generate_max_input(p, orig_bin, work, explicit_specs)
        if max_inp is None:
            out["perf"] = {"measured": False, "reason": max_label, "interactive": bool(p["interactor"])}
            return out

        if p["interactor"]:
            w_orig, st0 = run_interactive(p["interactor"], max_inp, [orig_bin], os.path.join(work, "mo0"), 15.0)
            w_port, st1 = run_interactive(p["interactor"], max_inp, [port_bin], os.path.join(work, "mo1"), 15.0)
            if st0 == "ok" and st1 == "ok":
                orig_ms = round(w_orig * 1000.0, 2)
                cpp_ms = round(w_port * 1000.0, 2)
                out["perf"] = {
                    "measured": True,
                    "interactive": True,
                    "input_source": max_label,
                    "input_bytes": os.path.getsize(max_inp),
                    "cpp_ms": cpp_ms,
                    "orig_ms": orig_ms,
                    "time_ratio": round(cpp_ms / orig_ms, 3) if orig_ms > 0 else None,
                    "cpp_rss_kb": None,
                    "orig_rss_kb": None,
                    "rss_ratio": None,
                    "output_match": True,
                }
            else:
                out["perf"] = {
                    "measured": False,
                    "interactive": True,
                    "input_source": max_label,
                    "reason": f"orig={st0}, port={st1}",
                }
            return out

        # Non-interactive alternating timing + peak RSS
        o_orig = os.path.join(work, "max_orig.out")
        o_port = os.path.join(work, "max_port.out")
        orig_wall_rss, orig_rss, st0 = run_timed([orig_bin], max_inp, o_orig, 15.0, rss=True)
        port_wall_rss, port_rss, st1 = run_timed([port_bin], max_inp, o_port, 15.0, rss=True)
        if st0 != "ok" or st1 != "ok":
            out["perf"] = {
                "measured": False,
                "interactive": False,
                "input_source": max_label,
                "reason": f"orig={st0}, port={st1}",
            }
            return out
        matched, _ = judge_output(p, max_inp, o_orig, o_port)
        orig_times = [orig_wall_rss]
        port_times = [port_wall_rss]
        for _ in range(max(0, runs - 1)):
            w1, _, s1 = run_timed([port_bin], max_inp, o_port, 15.0, rss=False)
            w0, _, s0 = run_timed([orig_bin], max_inp, o_orig, 15.0, rss=False)
            if s1 == "ok":
                port_times.append(w1)
            if s0 == "ok":
                orig_times.append(w0)
        cpp_ms = round(statistics.median(port_times) * 1000.0, 2)
        orig_ms = round(statistics.median(orig_times) * 1000.0, 2)
        out["perf"] = {
            "measured": True,
            "interactive": False,
            "input_source": max_label,
            "input_bytes": os.path.getsize(max_inp),
            "cpp_ms": cpp_ms,
            "orig_ms": orig_ms,
            "time_ratio": round(cpp_ms / orig_ms, 3) if orig_ms > 0 else None,
            "cpp_rss_kb": port_rss,
            "orig_rss_kb": orig_rss,
            "rss_ratio": round(port_rss / orig_rss, 3) if orig_rss > 0 else None,
            "output_match": matched,
        }
        if not matched:
            out["diff_ok"] = False
            out["diff_detail"] = f"max-size output mismatch ({max_label})"

        # 3. Apples-to-apples interpreter vs C++ (-O2) ratio on the same input.
        # Prefer max_inp when C++ finishes within 250ms (so interpreter has headroom);
        # otherwise fall back to the largest diff case.
        ratio_inp, ratio_src, ratio_cpp_ms = None, None, None
        if cpp_ms is not None and cpp_ms <= 250.0:
            ratio_inp, ratio_src, ratio_cpp_ms = max_inp, max_label, cpp_ms
        elif diff_cases:
            ratio_inp, ratio_src = max(diff_cases, key=lambda c: os.path.getsize(c[0]))
            w_c, _, s_c = run_timed([port_bin], ratio_inp, os.path.join(work, "icpp.out"), 10.0, rss=False)
            if s_c == "ok":
                ratio_cpp_ms = round(w_c * 1000.0, 2)
        if ratio_inp is not None and ratio_cpp_ms is not None and ratio_cpp_ms > 0:
            iw_r, _, st_r = run_timed(
                [cpl, "run", p["abs_path"]],
                ratio_inp,
                os.path.join(work, "iratio.out"),
                interp_timeout,
                rss=False,
            )
            if st_r == "ok":
                ims_r = round(iw_r * 1000.0, 2)
                out["interp"]["ratio_interp_ms"] = ims_r
                out["interp"]["ratio_cpp_ms"] = ratio_cpp_ms
                out["interp"]["ratio"] = round(ims_r / ratio_cpp_ms, 2)
                out["interp"]["ratio_source"] = ratio_src
            else:
                out["interp"]["ratio_error"] = f"{ratio_src}: {st_r}"
                out["interp"]["ratio_source"] = ratio_src
    return out


# ---------------------------------------------------------------------------
# Friction report summary over research/frictions/*.md
# ---------------------------------------------------------------------------

_FIELD_RE = re.compile(r"^\s*(?:[-*]\s*)?\*\*([^*:]+):?\*\*:?\s*(.*)$")
_NON_FRICTION_SECTIONS = {
    "measurements",
    "measurement",
    "metrics",
    "summary",
    "algorithm",
    "algorithm summary",
    "what worked",
    "what worked well",
    "verification",
    "notes",
    "context",
    "status",
}


_AREA_ALIASES = {
    "graphs": "graphs",
    "graph": "graphs",
    "std.graph": "graphs",
    "trees": "trees",
    "tree": "trees",
    "std.tree": "trees",
    "ranges": "ranges",
    "range": "ranges",
    "strings": "strings",
    "string": "strings",
    "std.str": "strings",
    "math": "math",
    "number theory": "math",
    "std.math": "math",
    "std.mod": "math",
    "std.poly": "math",
    "std.matrix": "math",
    "flows": "flows",
    "flow": "flows",
    "std.flow": "flows",
    "containers": "containers",
    "container": "containers",
    "geometry": "geometry",
    "geom": "geometry",
    "std.geometry": "geometry",
    "dp": "dp",
    "slope": "dp",
    "std.slope": "dp",
    "seq": "seq",
    "std.seq": "seq",
    "core": "core",
    "core language": "core",
    "codegen": "core",
    "interp": "core",
    "syntax": "core",
    "check": "core",
    "types": "core",
    "tooling": "core",
    "cli": "core",
    "lsp": "core",
    "fmt": "core",
    "lint": "core",
    "stress": "core",
    "io": "core",
    "i/o": "core",
}


def _parse_explicit_area(raw):
    """Map an explicit `Area:` field value (taking its primary/leading area) to a canonical area, or None."""
    if not raw or not raw.strip():
        return None
    # Take the leading clause before '+', '/', '(', ',', or ';' and strip backticks/whitespace.
    primary = re.split(r"[+/(,;]", raw, maxsplit=1)[0].strip().strip("`'\"").strip().lower()
    if primary in _AREA_ALIASES:
        return _AREA_ALIASES[primary]
    # Try first token as a fallback (e.g. "strings 4.13" or "core syntax").
    first = primary.split()[0] if primary.split() else ""
    return _AREA_ALIASES.get(first)


def _kw_matches(kw, hay):
    if kw.isalpha() and len(kw) <= 4:
        return re.search(r"\b" + re.escape(kw) + r"\b", hay) is not None
    return kw in hay


def route_friction_area(title, body, fields):
    """Route a friction item to one of the 10 stdlib areas (or 'core').

    Prefers the explicit `Area:` field (including decorated forms like
    ``seq (`plan/STDLIB.md`)``, ``containers (`plan/STDLIB.md`) + codegen``,
    ``number theory (`plan/STDLIB.md`) + core language``, or ``core language``)
    and falls back to keyword matching only when `Area:` is missing or unrecognized.
    """
    explicit = _parse_explicit_area(fields.get("area", ""))
    if explicit is not None:
        return explicit
    hay = (title + "\n" + body).lower()
    for area, kws in _AREA_KEYWORDS:
        if any(_kw_matches(kw, hay) for kw in kws):
            return area
    return "core"


def parse_friction_file(path):
    """Parse one research/frictions/<NAME>.md file into structured friction items."""
    text = _read_text(path)
    rel = os.path.relpath(path, ROOT) if os.path.isabs(path) else path
    items = []
    warnings = []
    raw_sections = []
    for sec in re.split(r"(?m)^##\s+", text)[1:]:
        lines = sec.splitlines()
        if not lines:
            continue
        title = lines[0].strip()
        low = title.lower()
        if low in _NON_FRICTION_SECTIONS or low.startswith("status:"):
            continue
        if low in ("frictions", "friction", "friction items"):
            for sub in re.split(r"(?m)^###\s+", "\n".join(lines[1:]))[1:]:
                sub_lines = sub.splitlines()
                if sub_lines:
                    raw_sections.append((sub_lines[0].strip(), sub_lines[1:]))
            continue
        raw_sections.append((title, lines[1:]))

    for title, body_lines in raw_sections:
        fields = {}
        cur_key = None
        for line in body_lines:
            m = _FIELD_RE.match(line)
            if m:
                key = m.group(1).strip().rstrip(":").lower()
                val = m.group(2).strip()
                fields[key] = val
                cur_key = key
            elif cur_key and line.startswith("  ") and not line.strip().startswith("```"):
                pass

        # Severity
        raw_sev = fields.get("severity", "").strip().lower()
        sev_match = re.match(r"^(blocker|high|medium|low)\b", raw_sev)
        severity = sev_match.group(1) if sev_match else "unknown"
        if severity == "unknown":
            warnings.append(f"{rel} :: {title}: missing or unrecognized Severity ({raw_sev!r})")

        # Status (default open; closed/fixed/resolved or ~~title~~ marks closed)
        raw_status = fields.get("status", "").strip().lower()
        if title.startswith("~~") and title.endswith("~~"):
            status = "closed"
        elif any(w in raw_status for w in ("closed", "fixed", "resolved", "done")):
            status = "closed"
        else:
            status = "open"

        # Goals: normalize to canonical AGENTS.md goal names; warn on missing/unknown
        raw_goals = fields.get("goal it hurts", fields.get("goals", fields.get("goal", "")))
        raw_clean = re.sub(r"\([^)]*\)", "", raw_goals).lower()
        canonical_map = [
            ("express", ("express", "revise", "revisable", "guessable")),
            ("hard-to-get-wrong", ("hard-to-get-wrong", "hard to get wrong", "safety", "wrong")),
            ("performance", ("performance", "control", "fast", "speed")),
            ("doubt-to-evidence", ("doubt-to-evidence", "doubt to evidence", "evidence", "diagnos")),
            ("dependable-submission", ("dependable-submission", "dependable submission", "submission", "judge")),
            ("accumulate", ("accumulate", "library", "reuse")),
            ("coherence", ("coherence", "coherent")),
        ]
        goals = []
        for cname, needles in canonical_map:
            if any(n in raw_clean for n in needles):
                goals.append(cname)
        if not goals and raw_clean.strip():
            warnings.append(f"{rel} :: {title}: unrecognized Goal it hurts ({raw_goals!r})")
        elif not goals:
            warnings.append(f"{rel} :: {title}: missing Goal it hurts")

        area = route_friction_area(title, "\n".join(body_lines), fields)

        items.append(
            {
                "file": rel,
                "title": title.strip("~").strip(),
                "severity": severity,
                "goals": goals,
                "area": area,
                "status": status,
                "general_fix": fields.get("general fix", "")[:200],
            }
        )
    return items, warnings


def summarize_frictions(frictions_dir=FRICTIONS_DIR):
    """Summarize all research/frictions/*.md reports (excluding README.md)."""
    files = []
    if os.path.isdir(frictions_dir):
        for path in sorted(glob.glob(os.path.join(frictions_dir, "*.md"))):
            if os.path.basename(path).lower() == "readme.md":
                continue
            files.append(path)

    all_items = []
    all_warnings = []
    for path in files:
        items, warnings = parse_friction_file(path)
        all_items.extend(items)
        all_warnings.extend(warnings)

    open_items = [it for it in all_items if it["status"] == "open"]
    closed_items = [it for it in all_items if it["status"] != "open"]

    by_severity = {s: 0 for s in (*VALID_SEVERITIES, "unknown")}
    by_goal = Counter()
    by_area = Counter()
    for it in open_items:
        by_severity[it["severity"]] = by_severity.get(it["severity"], 0) + 1
        for g in it["goals"]:
            by_goal[g] += 1
        by_area[it.get("area", "core")] += 1

    by_area_queues = {}
    for area in (*STDLIB_AREAS, "core"):
        aitems = [it for it in open_items if it.get("area", "core") == area]
        if aitems:
            by_area_queues[area] = {
                "open": len(aitems),
                "high": sum(1 for it in aitems if it["severity"] in ("blocker", "high")),
                "files": sorted({it["file"] for it in aitems}),
            }

    return {
        "files": len(files),
        "total": len(all_items),
        "open": len(open_items),
        "closed": len(closed_items),
        "by_severity": by_severity,
        "by_goal": dict(sorted(by_goal.items(), key=lambda kv: (-kv[1], kv[0]))),
        "by_area": {k: by_area.get(k, 0) for k in (*STDLIB_AREAS, "core")},
        "by_area_queues": by_area_queues,
        "open_blockers": [
            {"file": it["file"], "title": it["title"], "area": it.get("area", "core")}
            for it in open_items
            if it["severity"] == "blocker"
        ],
        "open_highs": [
            {"file": it["file"], "title": it["title"], "area": it.get("area", "core")}
            for it in open_items
            if it["severity"] == "high"
        ],
        "warnings": all_warnings,
        "items": all_items,
    }


def run_bug_replay_suite(cpl, suite_dir=BUG_REPLAY_DIR):
    """Run the P0 bug-replay suite (AGENTS.md goal 2) and classify each case."""
    manifest_path = os.path.join(suite_dir, "manifest.json")
    if not os.path.isfile(manifest_path):
        return None
    with open(manifest_path, encoding="utf-8") as f:
        manifest = json.load(f)
    bar_pct = float(manifest.get("bar_pct", BUG_REPLAY_BAR_PCT))
    manifest_cases = manifest.get("cases", [])

    def _eval_one(c):
        path = os.path.join(suite_dir, c["file"])
        chk = sh([cpl, "check", path], timeout=10)
        if chk.returncode != 0:
            outcome = "prevented"
        else:
            lnt = sh([cpl, "lint", path], timeout=10)
            lout = (lnt.stdout or "") + (lnt.stderr or "")
            if lnt.returncode != 0 or "warning[" in lout or "error[" in lout:
                outcome = "prevented"
            elif c.get("kind") == "complexity":
                outcome = "left_to_diagnose"
            else:
                run = sh([cpl, "run", path], input=c.get("input", ""), timeout=10)
                if run.returncode != 0:
                    outcome = "caught_on_run"
                else:
                    outcome = (
                        "left_to_diagnose"
                        if run.stdout != c.get("expected_output", "")
                        else "unexpected_pass"
                    )
        return {
            "id": c["id"],
            "bug_class": c["bug_class"],
            "outcome": outcome,
        }

    workers = min(8, max(1, os.cpu_count() or 1))
    with concurrent.futures.ThreadPoolExecutor(max_workers=workers) as ex:
        cases_out = list(ex.map(_eval_one, manifest_cases))

    counts = {"prevented": 0, "caught_on_run": 0, "left_to_diagnose": 0, "unexpected_pass": 0}
    by_class = {}
    for rec in cases_out:
        outcome = rec["outcome"]
        counts[outcome] = counts.get(outcome, 0) + 1
        cls = rec["bug_class"]
        d = by_class.setdefault(
            cls,
            {"total": 0, "prevented": 0, "caught_on_run": 0, "left_to_diagnose": 0, "unexpected_pass": 0},
        )
        d["total"] += 1
        d[outcome] = d.get(outcome, 0) + 1
    for d in by_class.values():
        h = d["prevented"] + d["caught_on_run"]
        d["handled"] = h
        d["handled_pct"] = round(100.0 * h / d["total"], 1) if d["total"] else 0.0

    total = len(cases_out)
    handled = counts["prevented"] + counts["caught_on_run"]
    handled_pct = round(100.0 * handled / total, 1) if total else 0.0
    return {
        "total": total,
        "prevented": counts["prevented"],
        "caught_on_run": counts["caught_on_run"],
        "left_to_diagnose": counts["left_to_diagnose"],
        "unexpected_pass": counts.get("unexpected_pass", 0),
        "handled": handled,
        "handled_pct": handled_pct,
        "bar_pct": bar_pct,
        "bar_ok": handled_pct >= bar_pct and counts.get("unexpected_pass", 0) == 0,
        "by_class": by_class,
        "cases": cases_out,
    }


SPELLING_DRIFT_SPECS = {
    # Canonical forms: decisions.md §20 for multitest, §53 for methods.
    # Free-form generators (`sum(f(x) for x in xs)`, `first(i for i in ... if ...)`)
    # stay canonical per §53.1 and are excluded from non-canonical counts.
    "multitest": {
        "canonical": "@!multitest",
        "patterns": {
            "@!multitest": re.compile(r"(?m)^\s*@!multitest\b"),
            "@multitest": re.compile(r"(?m)^\s*@multitest\b"),
            "--- multitest ---": re.compile(r"(?m)^\s*---\s*(?:with\s+)?multitest\b"),
        },
    },
    "sum": {
        "canonical": "xs.sum()",
        "skip_generators": {"sum(xs)"},
        "patterns": {
            "xs.sum()": re.compile(r"\.sum\s*\("),
            "sum(xs)": re.compile(r"(?<![.\w])(?<!fn )sum\s*\("),
        },
    },
    "first": {
        "canonical": "xs.first(...)",
        "skip_generators": {"first(...)"},
        "patterns": {
            "xs.first(...)": re.compile(r"\.first\s*\("),
            "first(...)": re.compile(r"(?<![.\w])(?<!fn )first\s*\("),
        },
    },
    "sort": {
        "canonical": "xs.sort(...)",
        "patterns": {
            "xs.sort(...)": re.compile(r"\.sort\s*\("),
            "sort(mut xs)": re.compile(r"(?<![.\w])(?<!fn )sort\s*\(\s*mut\b"),
        },
    },
    "where_filter": {
        "canonical": "if / positional",
        "patterns": {
            "if / positional": re.compile(r"\bfor\b[^\n]*\bif\b"),
            "where filter": re.compile(r"\bwhere\s*:|\bfor\b[^\n]*\bwhere\b"),
        },
    },
    "dot_pow": {
        "canonical": "a ** b",
        "patterns": {
            "a ** b": re.compile(r"\*\*"),
            ".pow(...)": re.compile(r"\.pow\s*\("),
        },
    },
    "getter_parens": {
        "canonical": ".getter",
        "patterns": {
            ".getter": re.compile(
                r"\.(?:len|is_empty|empty|size|rev|inv|last|front|back|top|keys|values|items|chars|bytes|popcount|bit_len)\b(?!\s*\()"
            ),
            ".getter()": re.compile(
                r"\.(?:len|is_empty|empty|size|rev|inv|last|front|back|top|keys|values|items|chars|bytes|popcount|bit_len)\s*\(\s*\)"
            ),
        },
    },
    "turbofish": {
        "canonical": "f<T>(...)",
        "patterns": {
            "f<T>(...)": re.compile(r"\b[a-z_][a-zA-Z0-9_]*<[A-Z_][A-Za-z0-9_<>, \.]*>\s*\("),
            "f::<T>(...)": re.compile(r"\b[a-zA-Z_][a-zA-Z0-9_]*::<\s*[^>\n]+\s*>"),
        },
    },
}


def _call_args_slice(src, open_paren_end):
    depth = 1
    i = open_paren_end
    n = len(src)
    while i < n and depth > 0:
        ch = src[i]
        if ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
        i += 1
    return src[open_paren_end : max(open_paren_end, i - 1)]


def _count_spelling_matches(clean, pat, skip_generator=False):
    if not skip_generator:
        return len(pat.findall(clean))
    count = 0
    for m in pat.finditer(clean):
        args = _call_args_slice(clean, m.end())
        if re.search(r"\bfor\b", args):
            continue
        count += 1
    return count

_RETIRED_PORT_DIR_PREFIXES = (
    "scout_batch_",
    "blind_critic_",
    "hillclimb_",
    "sim3",
    "before",
    "wave1",
    "wave2",
)


def _strip_comments_and_strings(src):
    out = []
    for line in src.splitlines():
        line = re.sub(r'"(?:\\.|[^"\\])*"', '""', line)
        line = re.sub(r"//.*", "", line)
        out.append(line)
    return "\n".join(out)


def _is_retired_port_path(rel_path):
    parts = rel_path.replace("\\", "/").split("/")
    return any(
        part == pref or part.startswith(pref)
        for part in parts
        for pref in _RETIRED_PORT_DIR_PREFIXES
    )


def measure_spelling_drift(ports_root=os.path.join(ROOT, "proto", "ports")):
    """Count canonical vs non-canonical spellings across proto/ports/**/*.cp."""
    if not os.path.isdir(ports_root):
        return None
    constructs = {}
    for name, spec in SPELLING_DRIFT_SPECS.items():
        constructs[name] = {
            "canonical": spec["canonical"],
            "counts": {k: 0 for k in spec["patterns"]},
            "active_counts": {k: 0 for k in spec["patterns"]},
            "non_canonical": 0,
            "active_non_canonical": 0,
        }

    files_scanned = 0
    active_files_scanned = 0
    for dirpath, _dirnames, filenames in os.walk(ports_root):
        for fn in sorted(filenames):
            if not fn.endswith(".cp"):
                continue
            full = os.path.join(dirpath, fn)
            rel = os.path.relpath(full, ports_root)
            active = not _is_retired_port_path(rel)
            files_scanned += 1
            if active:
                active_files_scanned += 1
            clean = _strip_comments_and_strings(_read_text(full))
            for name, spec in SPELLING_DRIFT_SPECS.items():
                entry = constructs[name]
                skip_gens = spec.get("skip_generators") or set()
                for spelling, pat in spec["patterns"].items():
                    n = _count_spelling_matches(clean, pat, skip_generator=(spelling in skip_gens))
                    if n:
                        entry["counts"][spelling] += n
                        if active:
                            entry["active_counts"][spelling] += n

    total_non_canonical = 0
    active_non_canonical = 0
    for name, entry in constructs.items():
        canon = entry["canonical"]
        nc = sum(v for k, v in entry["counts"].items() if k != canon)
        anc = sum(v for k, v in entry["active_counts"].items() if k != canon)
        entry["non_canonical"] = nc
        entry["active_non_canonical"] = anc
        total_non_canonical += nc
        active_non_canonical += anc

    return {
        "files_scanned": files_scanned,
        "active_files_scanned": active_files_scanned,
        "total_non_canonical": total_non_canonical,
        "active_non_canonical": active_non_canonical,
        "constructs": constructs,
    }


def run_revision_suite(cpl, suite_dir=REVISION_DIR):
    """Run the unseen-problem revision suite (AGENTS.md goal 1) and measure lines touched."""
    manifest_path = os.path.join(suite_dir, "manifest.json")
    if not os.path.isfile(manifest_path):
        return None
    with open(manifest_path, encoding="utf-8") as f:
        manifest = json.load(f)
    cases_out = []
    for c in manifest.get("cases", []):
        v1_path = os.path.join(suite_dir, c["v1"])
        v2_path = os.path.join(suite_dir, c["v2"])
        ok_all = True
        detail = "ok"
        for tag in ("v1", "v2"):
            p = os.path.join(suite_dir, c[tag])
            inp = _read_text(os.path.join(suite_dir, c[f"{tag}_in"]))
            exp = _read_text(os.path.join(suite_dir, c[f"{tag}_out"]))
            r = sh([cpl, "run", p], input=inp, timeout=10)
            if r.returncode != 0 or r.stdout != exp:
                ok_all = False
                detail = f"{tag} failed (rc={r.returncode})"
                break
        a_lines = _read_text(v1_path).splitlines()
        b_lines = _read_text(v2_path).splitlines()
        diff = list(difflib.unified_diff(a_lines, b_lines, n=0))
        touched = sum(
            1
            for l in diff
            if (l.startswith("+") or l.startswith("-"))
            and not l.startswith("+++")
            and not l.startswith("---")
        )
        hunks = sum(1 for l in diff if l.startswith("@@"))
        cases_out.append(
            {
                "id": c["id"],
                "ok": ok_all,
                "detail": detail,
                "lines_touched": touched,
                "hunks": hunks,
            }
        )
    total_touched = sum(x["lines_touched"] for x in cases_out)
    return {
        "total": len(cases_out),
        "passed": sum(1 for x in cases_out if x["ok"]),
        "total_lines_touched": total_touched,
        "cases": cases_out,
    }


def run_friction_lint(ports):
    """Run scripts/friction_lint.py in-process over the discovered port files."""
    scripts_dir = os.path.dirname(os.path.abspath(__file__))
    if scripts_dir not in sys.path:
        sys.path.insert(0, scripts_dir)
    try:
        import friction_lint as fl

        by_sev = Counter()
        by_rule = Counter()
        for p in ports:
            raw = _read_text(p["abs_path"])
            code = fl.strip_code(raw)
            lines = code.split("\n")
            raw_lines = raw.split("\n")
            for key, sev, _desc, _src, fn in fl.RULES:
                hits = fn(code, lines, raw_lines)
                if hits:
                    by_sev[sev] += len(hits)
                    by_rule[key] += len(hits)
        return {
            "files": len(ports),
            "wa_trap": by_sev["wa_trap"],
            "real_gap": by_sev["real_gap"],
            "stale_port": by_sev["stale_port"],
            "by_rule": dict(sorted(by_rule.items(), key=lambda kv: (-kv[1], kv[0]))),
        }
    except Exception as e:
        return {"files": len(ports), "wa_trap": 0, "real_gap": 0, "stale_port": 0, "error": str(e)}


# ---------------------------------------------------------------------------
# Percentiles, Totals, and Baseline Diff
# ---------------------------------------------------------------------------

def percentile(xs, q):
    """Nearest-rank percentile for q in [0, 100]."""
    if not xs:
        return None
    s = sorted(xs)
    idx = min(len(s) - 1, max(0, math.ceil(len(s) * q / 100.0) - 1))
    return s[idx]


def summarize_dream_compile(dreams_rep):
    """Compute share of dream/*.cp files that pass `cpl check` with zero errors."""
    if not dreams_rep or not dreams_rep.get("dreams"):
        return None
    total = int(dreams_rep["dreams"])
    with_errors = int(dreams_rep.get("with_errors", 0))
    compiled = max(0, total - with_errors)
    return {
        "compiled": compiled,
        "total": total,
        "rate_pct": round(100.0 * compiled / total, 1) if total else 0.0,
    }


def compute_totals(
    port_records,
    fl_summary,
    fr_summary,
    bug_replay=None,
    revision=None,
    spelling_drift=None,
    dream_scores=None,
    dream_compile=None,
    gate_fast=None,
    taste_summary=None,
):
    total = len(port_records)
    passed = sum(1 for r in port_records if r["gate"] == "pass")
    failed = total - passed

    cp_toks = [r["cp_tokens"] for r in port_records]
    orig_toks = [r["orig_tokens"] for r in port_records]
    stripped_toks = [r["orig_stripped_tokens"] for r in port_records]
    raw_ratios = [r["token_ratio"] for r in port_records if r["token_ratio"] is not None]
    stripped_ratios = [r["stripped_token_ratio"] for r in port_records if r["stripped_token_ratio"] is not None]

    emitted = [(r["emitted_bytes"], r["id"]) for r in port_records if r["emitted_bytes"] is not None]
    emitted_vals = [b for b, _ in emitted]
    max_emitted, max_emitted_port = max(emitted, default=(None, None))

    perf_recs = [r for r in port_records if (r.get("perf") or {}).get("measured")]
    time_ratios = [
        (r["perf"]["time_ratio"], r["id"])
        for r in perf_recs
        if r["perf"].get("time_ratio") is not None
        and max(r["perf"].get("cpp_ms") or 0, r["perf"].get("orig_ms") or 0) >= PERF_FLOOR_MS
    ]
    all_time_ratios = [
        (r["perf"]["time_ratio"], r["id"])
        for r in perf_recs
        if r["perf"].get("time_ratio") is not None
    ]
    t_use = time_ratios if time_ratios else all_time_ratios
    t_vals = [v for v, _ in t_use]
    t_max, t_max_port = max(t_use, default=(None, None))
    t_p50 = round(statistics.median(t_vals), 3) if t_vals else None
    t_max_r = round(t_max, 3) if t_max is not None else None

    perf_by_id = {r["id"]: r["perf"] for r in perf_recs}
    worst_10 = [
        {
            "id": pid,
            "time_ratio": round(ratio, 3),
            "cpp_ms": perf_by_id[pid].get("cpp_ms"),
            "orig_ms": perf_by_id[pid].get("orig_ms"),
            "input_source": perf_by_id[pid].get("input_source"),
        }
        for ratio, pid in sorted(t_use, key=lambda x: (-x[0], x[1]))[:10]
    ]

    dream_recs = [r for r in port_records if r.get("dream_tokens") is not None]
    dream_cp_total = sum(r["cp_tokens"] for r in dream_recs)
    dream_total = sum(r["dream_tokens"] for r in dream_recs)
    dream_ratios = [(r["dream_ratio"], r["id"]) for r in dream_recs if r.get("dream_ratio") is not None]
    dr_vals = [v for v, _ in dream_ratios]
    dr_max, dr_max_port = max(dream_ratios, default=(None, None))

    rss_ratios = [
        (r["perf"]["rss_ratio"], r["id"])
        for r in perf_recs
        if r["perf"].get("rss_ratio") is not None
    ]
    r_vals = [v for v, _ in rss_ratios]
    r_max, r_max_port = max(rss_ratios, default=(None, None))

    interp_recs = [r for r in port_records if (r.get("interp") or {}).get("ms") is not None]
    i_vals = [(r["interp"]["ms"], r["id"]) for r in interp_recs]
    i_ms = [v for v, _ in i_vals]
    i_max, i_max_port = max(i_vals, default=(None, None))

    i_ratios = [
        (r["interp"]["ratio"], r["id"])
        for r in port_records
        if (r.get("interp") or {}).get("ratio") is not None
        and ((r["interp"].get("ratio_interp_ms") or 0) >= INTERP_FLOOR_MS)
    ]
    all_i_ratios = [
        (r["interp"]["ratio"], r["id"])
        for r in port_records
        if (r.get("interp") or {}).get("ratio") is not None
    ]
    ir_use = i_ratios if i_ratios else all_i_ratios
    ir_vals = [v for v, _ in ir_use]
    ir_p50 = round(statistics.median(ir_vals), 2) if ir_vals else None
    ir_p95 = round(percentile(ir_vals, 95), 2) if ir_vals else None
    ir_max, ir_max_port = max(ir_use, default=(None, None))
    interp_by_id = {r["id"]: (r.get("interp") or {}) for r in port_records}
    interp_worst_10 = [
        {
            "id": pid,
            "ratio": round(ratio, 2),
            "interp_ms": interp_by_id[pid].get("ratio_interp_ms"),
            "cpp_ms": interp_by_id[pid].get("ratio_cpp_ms"),
            "source": interp_by_id[pid].get("ratio_source"),
        }
        for ratio, pid in sorted(ir_use, key=lambda x: (-x[0], x[1]))[:10]
    ]

    if taste_summary is None:
        taste_scores = [r["taste_score"] for r in port_records if r.get("taste_score") is not None]
        med_t = statistics.median(taste_scores) if taste_scores else None
        if isinstance(med_t, float) and med_t.is_integer():
            med_t = int(med_t)
        taste_summary = {
            "reviewed": len(taste_scores),
            "pending": 0,
            "median": round(med_t, 2) if isinstance(med_t, float) else med_t,
            "under_4": sum(1 for s in taste_scores if s < 4),
            "min": min(taste_scores) if taste_scores else None,
            "max": max(taste_scores) if taste_scores else None,
            "histogram": {str(k): sum(1 for s in taste_scores if int(round(s)) == k) for k in range(1, 6)},
            "stale": 0,
            "unreviewed": sum(1 for r in port_records if r.get("taste_score") is None),
        }

    return {
        "ports_total": total,
        "ports_passed": passed,
        "ports_failed": failed,
        "interactive_ports": sum(1 for r in port_records if r.get("interactive")),
        "checker_ports": sum(1 for r in port_records if r.get("has_checker")),
        "tokens": {
            "cp_total": sum(cp_toks),
            "orig_total": sum(orig_toks),
            "orig_stripped_total": sum(stripped_toks),
            "ratio_of_totals": round(sum(cp_toks) / sum(orig_toks), 4) if sum(orig_toks) else None,
            "stripped_ratio_of_totals": round(sum(cp_toks) / sum(stripped_toks), 4) if sum(stripped_toks) else None,
            "median_ratio": round(statistics.median(raw_ratios), 4) if raw_ratios else None,
            "p90_ratio": round(percentile(raw_ratios, 90), 4) if raw_ratios else None,
            "max_ratio": round(max(raw_ratios), 4) if raw_ratios else None,
            "median_stripped_ratio": round(statistics.median(stripped_ratios), 4) if stripped_ratios else None,
            "p90_stripped_ratio": round(percentile(stripped_ratios, 90), 4) if stripped_ratios else None,
            "max_stripped_ratio": round(max(stripped_ratios), 4) if stripped_ratios else None,
            "over_1_0x_raw": sum(1 for x in raw_ratios if x > 1.0),
            "over_1_0x_stripped": sum(1 for x in stripped_ratios if x > 1.0),
        },
        "dream": {
            "ports": len(dream_recs),
            "cp_total": dream_cp_total,
            "dream_total": dream_total,
            "ratio_of_totals": round(dream_cp_total / dream_total, 4) if dream_total else None,
            "median_ratio": round(statistics.median(dr_vals), 4) if dr_vals else None,
            "p90_ratio": round(percentile(dr_vals, 90), 4) if dr_vals else None,
            "max_ratio": round(dr_max, 4) if dr_max is not None else None,
            "max_port": dr_max_port,
        },
        "emitted_bytes": {
            "measured": len(emitted_vals),
            "median": int(statistics.median(emitted_vals)) if emitted_vals else None,
            "p95": percentile(emitted_vals, 95),
            "max": max_emitted,
            "max_port": max_emitted_port,
            "over_48kb": sum(1 for b in emitted_vals if b > SIZE_WARN),
            "over_64kb": sum(1 for b in emitted_vals if b > SIZE_LIMIT),
        },
        "perf": {
            "measured": len(perf_recs),
            "above_floor": len(time_ratios),
            "floor_ms": PERF_FLOOR_MS,
            "mismatches": sum(1 for r in perf_recs if not r["perf"].get("output_match", True)),
            "time_p50": t_p50,
            "time_p95": round(percentile(t_vals, 95), 3) if t_vals else None,
            "time_max": t_max_r,
            "time_max_port": t_max_port,
            "time_p50_goal": TIME_P50_GOAL,
            "time_max_goal": TIME_MAX_GOAL,
            "p50_ok": (t_p50 <= TIME_P50_GOAL) if t_p50 is not None else None,
            "max_ok": (t_max_r <= TIME_MAX_GOAL) if t_max_r is not None else None,
            "over_1_5x": sum(1 for v in t_vals if v > TIME_MAX_GOAL),
            "worst_10": worst_10,
            "rss_p50": round(statistics.median(r_vals), 3) if r_vals else None,
            "rss_p95": round(percentile(r_vals, 95), 3) if r_vals else None,
            "rss_max": round(r_max, 3) if r_max is not None else None,
            "rss_max_port": r_max_port,
        },
        "taste": taste_summary,
        "taste_stale": taste_summary.get("stale", 0),
        "taste_unreviewed": taste_summary.get("unreviewed", 0),
        "interp": {
            "measured": len(interp_recs),
            "passed": sum(1 for r in port_records if (r.get("interp") or {}).get("ok") is True),
            "failed": sum(1 for r in port_records if (r.get("interp") or {}).get("ok") is False),
            "median_ms": round(statistics.median(i_ms), 2) if i_ms else None,
            "p95_ms": round(percentile(i_ms, 95), 2) if i_ms else None,
            "max_ms": round(i_max, 2) if i_max is not None else None,
            "max_port": i_max_port,
            "ratio_measured": len(all_i_ratios),
            "ratio_above_floor": len(i_ratios),
            "ratio_p50": ir_p50,
            "ratio_p95": ir_p95,
            "ratio_max": round(ir_max, 2) if ir_max is not None else None,
            "ratio_max_port": ir_max_port,
            "ratio_bar": INTERP_RATIO_BAR,
            "ratio_north_star": INTERP_RATIO_NORTH_STAR,
            "bar_ok": (ir_p50 <= INTERP_RATIO_BAR) if ir_p50 is not None else None,
            "north_star_ok": (ir_p50 <= INTERP_RATIO_NORTH_STAR) if ir_p50 is not None else None,
            "over_10x": sum(1 for v in ir_vals if v > INTERP_RATIO_BAR),
            "worst_10": interp_worst_10,
        },
        "taste_lints": {
            "total": sum(r.get("taste_lints", 0) for r in port_records),
            "ports": sum(1 for r in port_records if r.get("taste_lints")),
            "by_lint": taste_by_lint(port_records),
        },
        "friction_lint": {
            "files": fl_summary.get("files", 0),
            "wa_trap": fl_summary.get("wa_trap", 0),
            "real_gap": fl_summary.get("real_gap", 0),
            "stale_port": fl_summary.get("stale_port", 0),
        },
        "frictions": {
            "files": fr_summary["files"],
            "total": fr_summary["total"],
            "open": fr_summary["open"],
            "closed": fr_summary["closed"],
            "by_severity": fr_summary["by_severity"],
            "by_goal": fr_summary["by_goal"],
            "by_area": fr_summary.get("by_area", {}),
            "open_blockers": len(fr_summary["open_blockers"]),
            "open_highs": len(fr_summary["open_highs"]),
            "warnings": len(fr_summary.get("warnings", [])),
        },
        "bug_replay": bug_replay,
        "revision": revision,
        "spelling_drift": spelling_drift,
        "dream_scores": dream_scores,
        "dream_compile": dream_compile,
        "gate_fast": gate_fast,
    }


def detect_change_scope(repo_root=ROOT, base_ref="origin/main"):
    """Classify the current change (staged + unstaged + ahead of base_ref) for gate attribution."""
    changed = set()
    try:
        ref_ok = subprocess.run(
            ["git", "rev-parse", "--verify", "--quiet", base_ref],
            cwd=repo_root,
            capture_output=True,
            text=True,
        ).returncode == 0
        diff_ref = base_ref if ref_ok else "HEAD"
        r = subprocess.run(
            ["git", "diff", "--name-only", diff_ref],
            cwd=repo_root,
            capture_output=True,
            text=True,
        )
        if r.returncode == 0:
            changed.update(line.strip() for line in r.stdout.splitlines() if line.strip())
        st = subprocess.run(
            ["git", "status", "--porcelain"],
            cwd=repo_root,
            capture_output=True,
            text=True,
        )
        if st.returncode == 0:
            for line in st.stdout.splitlines():
                if len(line) > 3:
                    p = line[3:].strip()
                    if " -> " in p:
                        p = p.split(" -> ", 1)[1]
                    if p:
                        changed.add(p)
    except OSError:
        return None

    touched_ports = set()
    reviewed_ports = set()
    codegen_causes = []
    touched_frictions = False
    touched_reviews = False
    touched_dreams = False
    touched_goal_tests = False
    for p in sorted(changed):
        norm = p.replace("\\", "/")
        if norm.startswith("crates/") or norm.startswith("stdlib/"):
            codegen_causes.append(norm)
        if norm.startswith("research/frictions/"):
            touched_frictions = True
        if norm.startswith("research/reviews/"):
            touched_reviews = True
            if norm.endswith(".md"):
                reviewed_ports.add(norm[len("research/reviews/"):-3])
        if norm.startswith("bench/goal_tests/"):
            touched_goal_tests = True
        if "/dream/" in norm and norm.startswith("proto/ports/"):
            touched_dreams = True
        pid = _port_id_from_relpath(norm)
        if pid:
            touched_ports.add(pid)
    return {
        "changed_files": changed,
        "touched_ports": touched_ports,
        "reviewed_ports": reviewed_ports,
        "touched_codegen_or_stdlib": bool(codegen_causes),
        "codegen_causes": codegen_causes,
        "touched_frictions": touched_frictions,
        "touched_reviews": touched_reviews,
        "touched_dreams": touched_dreams,
        "touched_goal_tests": touched_goal_tests,
    }


def _is_reviewed_rewrite(scope, pid):
    if not scope or pid not in scope.get("touched_ports", set()):
        return False
    return pid.rsplit("/", 1)[-1] in scope.get("reviewed_ports", set())


def _attribution_cause(scope, pid=None, area=None):
    """Return (attributable: bool, cause_tag: str) for a regression under `scope`."""
    if scope is None:
        return True, ""
    if pid is not None:
        if pid in scope.get("touched_ports", set()):
            return True, " [cause: touched port]"
        if scope.get("touched_codegen_or_stdlib"):
            causes = scope.get("codegen_causes") or []
            sample = ", ".join(causes[:3]) + (f" (+{len(causes) - 3} more)" if len(causes) > 3 else "")
            return True, f" [cause: stdlib/codegen diff ({sample})]"
        return False, " [unrelated to this change]"
    if area == "frictions":
        return (True, "") if scope.get("touched_frictions") else (False, " [unrelated to this change]")
    if area == "reviews":
        return (True, "") if scope.get("touched_reviews") else (False, " [unrelated to this change]")
    if area == "dream_compile":
        if scope.get("touched_codegen_or_stdlib") or scope.get("touched_dreams"):
            return True, ""
        return False, " [unrelated to this change]"
    if area == "spelling_drift":
        if scope.get("touched_ports") or scope.get("touched_dreams"):
            return True, ""
        return False, " [unrelated to this change]"
    if area == "goal_tests":
        if scope.get("touched_codegen_or_stdlib") or scope.get("touched_goal_tests"):
            return True, ""
        return False, " [unrelated to this change]"
    if area == "ports_or_codegen":
        if scope.get("touched_ports") or scope.get("touched_codegen_or_stdlib"):
            return True, ""
        return False, " [unrelated to this change]"
    return True, ""


def diff_against_baseline(current, baseline, mode="fast", subset=False, scope=None):
    """Compare current scoreboard against baseline and return structured diff + regressions."""
    regressions = []
    improvements = []
    warnings = []

    def record_regression(msg, pid=None, area=None):
        attr, tag = _attribution_cause(scope, pid=pid, area=area)
        if attr:
            regressions.append(msg + tag)
        else:
            warnings.append(msg + tag)

    cur_ports = {r["id"]: r for r in current["ports"]}
    base_ports = {r["id"]: r for r in baseline.get("ports", [])} if baseline else {}

    added = sorted(set(cur_ports) - set(base_ports))
    removed = [] if subset else sorted(set(base_ports) - set(cur_ports))

    # Hard invariants (even without baseline)
    for pid, r in sorted(cur_ports.items()):
        if r["gate"] != "pass":
            was = base_ports.get(pid, {}).get("gate")
            if was == "pass":
                record_regression(f"gate regression: {pid} (pass -> fail: {r.get('gate_detail')})", pid=pid)
            elif pid not in base_ports:
                record_regression(f"new port fails gate: {pid} ({r.get('gate_detail')})", pid=pid)
            else:
                record_regression(f"port fails gate: {pid} ({r.get('gate_detail')})", pid=pid)
        elif base_ports.get(pid, {}).get("gate") == "fail":
            improvements.append(f"gate fixed: {pid}")

        eb = r.get("emitted_bytes")
        if eb is not None and eb > SIZE_LIMIT:
            record_regression(f"emitted C++ over 64 KB: {pid} ({eb} B)", pid=pid)
        if pid in base_ports and eb is not None:
            prev_eb = base_ports[pid].get("emitted_bytes")
            if prev_eb is not None and eb > prev_eb + EMITTED_GROWTH_BYTES and eb > prev_eb * EMITTED_GROWTH_RATIO:
                msg = f"emitted C++ grew >5%: {pid} ({prev_eb} -> {eb} B, +{eb - prev_eb} B)"
                if _is_reviewed_rewrite(scope, pid) and eb <= SIZE_WARN:
                    # Accepting the growth means rebaselining in the same
                    # commit: a waiver that leaves the old baseline blames
                    # every later compiler or stdlib change for it.
                    regressions.append(
                        msg + f" [reviewed rewrite: accept it with `python3 scripts/scoreboard.py --update {pid}`]"
                    )
                else:
                    record_regression(msg, pid=pid)
            elif prev_eb is not None and prev_eb > eb + EMITTED_GROWTH_BYTES and eb < prev_eb * 0.95:
                improvements.append(f"emitted C++ shrank: {pid} ({prev_eb} -> {eb} B)")

        if pid in base_ports:
            b_dr = base_ports[pid].get("dream_ratio")
            c_dr = r.get("dream_ratio")
            dream_rewritten = base_ports[pid].get("dream_tokens") != r.get("dream_tokens")
            if b_dr is not None and c_dr is not None and dream_rewritten:
                # A rewritten dream is a new target; the ratchet restarts from it.
                improvements.append(f"dream rewritten: {pid} ({b_dr:.4f}x -> {c_dr:.4f}x)")
            elif b_dr is not None and c_dr is not None:
                if c_dr > b_dr + 1e-6:
                    record_regression(
                        f"dream gap regressed: {pid} ({b_dr:.4f}x -> {c_dr:.4f}x, "
                        f"cp={r.get('cp_tokens')}/dream={r.get('dream_tokens')})",
                        pid=pid,
                    )
                elif c_dr < b_dr - 0.01:
                    improvements.append(f"dream gap shrank: {pid} ({b_dr:.4f}x -> {c_dr:.4f}x)")

        if mode == "full" and pid in base_ports:
            bp = base_ports[pid].get("perf") or {}
            cp = r.get("perf") or {}
            if bp.get("measured") and bp.get("output_match") and cp.get("measured") and not cp.get("output_match"):
                record_regression(f"max-size output mismatch: {pid}", pid=pid)
            if (
                bp.get("measured")
                and cp.get("measured")
                and bp.get("time_ratio") is not None
                and cp.get("time_ratio") is not None
                and max(cp.get("cpp_ms") or 0, cp.get("orig_ms") or 0) >= PERF_FLOOR_MS
                and max(bp.get("cpp_ms") or 0, bp.get("orig_ms") or 0) >= PERF_FLOOR_MS
            ):
                br = bp["time_ratio"]
                cr = cp["time_ratio"]
                if cr > br * PERF_REGRESSION_RATIO:
                    record_regression(
                        f"runtime regressed >10%: {pid} ({br:.3f}x -> {cr:.3f}x, "
                        f"{cp.get('cpp_ms')}ms/{cp.get('orig_ms')}ms)",
                        pid=pid,
                    )
                elif cr < br * 0.90:
                    improvements.append(f"runtime improved: {pid} ({br:.3f}x -> {cr:.3f}x)")
            bi = base_ports[pid].get("interp") or {}
            ci = r.get("interp") or {}
            if (
                bi.get("ratio") is not None
                and ci.get("ratio") is not None
                and (ci.get("ratio_interp_ms") or 0) >= INTERP_FLOOR_MS
                and (bi.get("ratio_interp_ms") or 0) >= INTERP_FLOOR_MS
            ):
                bir, cir = bi["ratio"], ci["ratio"]
                if cir > bir * PERF_REGRESSION_RATIO:
                    record_regression(
                        f"interp ratio regressed >10%: {pid} ({bir:.2f}x -> {cir:.2f}x, "
                        f"{ci.get('ratio_interp_ms')}ms/{ci.get('ratio_cpp_ms')}ms)",
                        pid=pid,
                    )
                elif cir < bir * 0.90:
                    improvements.append(f"interp ratio improved: {pid} ({bir:.2f}x -> {cir:.2f}x)")

    # Aggregate checks (skip subset runs where totals aren't comparable)
    ct = current["totals"]
    bt = baseline.get("totals", {}) if baseline else {}

    if ct["friction_lint"]["wa_trap"] > 0:
        record_regression(f"friction_lint wa_trap > 0 ({ct['friction_lint']['wa_trap']})", area="ports_or_codegen")

    br = ct.get("bug_replay")
    if br is not None:
        if br.get("unexpected_pass", 0) > 0:
            record_regression(f"bug-replay suite has {br['unexpected_pass']} unexpected_pass case(s)", area="goal_tests")
        if not br.get("bar_ok"):
            record_regression(
                f"bug-replay handled {br.get('handled_pct')}% < {br.get('bar_pct')}% bar",
                area="goal_tests",
            )
        if baseline and not subset and bt.get("bug_replay"):
            bbr = bt["bug_replay"]
            if br["handled_pct"] < bbr.get("handled_pct", 0):
                record_regression(
                    f"bug-replay handled regressed: {bbr['handled_pct']}% -> {br['handled_pct']}%",
                    area="goal_tests",
                )
            rank = {"prevented": 2, "caught_on_run": 1, "left_to_diagnose": 0, "unexpected_pass": -1}
            b_cases = {c["id"]: c["outcome"] for c in bbr.get("cases", [])}
            for c in br.get("cases", []):
                prev = b_cases.get(c["id"])
                if prev is not None and rank.get(c["outcome"], -1) < rank.get(prev, -1):
                    record_regression(
                        f"bug-replay case regressed: {c['id']} ({prev} -> {c['outcome']})",
                        area="goal_tests",
                    )

    rv = ct.get("revision")
    if rv is not None:
        for c in rv.get("cases", []):
            if not c.get("ok"):
                record_regression(f"revision test failed: {c['id']} ({c.get('detail')})", area="goal_tests")
        if baseline and not subset and bt.get("revision"):
            b_rev = {c["id"]: c["lines_touched"] for c in bt["revision"].get("cases", [])}
            for c in rv.get("cases", []):
                prev = b_rev.get(c["id"])
                if prev is not None and c["lines_touched"] > prev:
                    record_regression(
                        f"revision lines_touched grew: {c['id']} ({prev} -> {c['lines_touched']})",
                        area="goal_tests",
                    )

    if baseline and not subset:
        base_blockers = bt.get("frictions", {}).get("open_blockers", 0)
        cur_blockers = ct["frictions"]["open_blockers"]
        if cur_blockers > base_blockers:
            record_regression(f"open blocker frictions increased: {base_blockers} -> {cur_blockers}", area="frictions")

        base_warn = bt.get("frictions", {}).get("warnings", 0)
        cur_warn = ct["frictions"].get("warnings", 0)
        if cur_warn > base_warn:
            record_regression(f"friction metadata warnings grew: {base_warn} -> {cur_warn}", area="frictions")

        bsd = bt.get("spelling_drift")
        csd = ct.get("spelling_drift")
        zero_locked = {"multitest", "sum", "first", "sort"}
        if csd:
            b_cs = (bsd.get("constructs") or {}) if bsd else {}
            c_cs = csd.get("constructs") or {}
            for cname, cent in c_cs.items():
                bent = b_cs.get(cname)
                c_ca = cent.get("active_non_canonical", 0)
                if cname in zero_locked or (bent is not None and bent.get("active_non_canonical", 0) == 0):
                    if c_ca > 0:
                        record_regression(
                            f"active/dream spelling drift ({cname}) must be 0, found {c_ca} "
                            "(run `cpl lint --fix` and `python3 scripts/fix-spelling-drift.py --write`)",
                            area="spelling_drift",
                        )
                elif bent is not None:
                    b_ca = bent.get("active_non_canonical", 0)
                    if c_ca > b_ca:
                        record_regression(
                            f"active/dream spelling drift ({cname}) grew: {b_ca} -> {c_ca}",
                            area="spelling_drift",
                        )
                    elif c_ca < b_ca:
                        improvements.append(
                            f"active/dream spelling drift ({cname}) reduced: {b_ca} -> {c_ca}"
                        )
        if bsd and csd:
            b_cs = bsd.get("constructs") or {}
            c_cs = csd.get("constructs") or {}
            shared = [n for n in c_cs if n in b_cs]
            b_tot = sum(b_cs[n]["non_canonical"] for n in shared)
            c_tot = sum(c_cs[n]["non_canonical"] for n in shared)
            if c_tot > b_tot:
                record_regression(f"spelling drift non-canonical total grew: {b_tot} -> {c_tot}", area="spelling_drift")
            elif c_tot < b_tot:
                improvements.append(f"spelling drift reduced: {b_tot} -> {c_tot}")
            for cname in shared:
                bent = b_cs[cname]
                cent = c_cs[cname]
                if cent["non_canonical"] > bent["non_canonical"]:
                    record_regression(
                        f"spelling drift ({cname}) grew: {bent['non_canonical']} -> {cent['non_canonical']}",
                        area="spelling_drift",
                    )
                elif cent["non_canonical"] < bent["non_canonical"]:
                    improvements.append(
                        f"spelling drift ({cname}) reduced: {bent['non_canonical']} -> {cent['non_canonical']}"
                    )

        bds = bt.get("dream_scores")
        cds = ct.get("dream_scores")
        if bds and cds and bds.get("scored", 0) > 0 and cds.get("scored", 0) > 0:
            b_med = bds.get("median")
            c_med = cds.get("median")
            if b_med is not None and c_med is not None:
                if c_med < b_med:
                    record_regression(f"dream-score median regressed: {b_med} -> {c_med}", area="reviews")
                elif c_med > b_med:
                    improvements.append(f"dream-score median improved: {b_med} -> {c_med}")
            b_u4 = bds.get("under_4", 0)
            c_u4 = cds.get("under_4", 0)
            if c_u4 > b_u4:
                record_regression(f"dream-score under-4 count grew: {b_u4} -> {c_u4}", area="reviews")
            elif c_u4 < b_u4:
                improvements.append(f"dream-score under-4 count shrank: {b_u4} -> {c_u4}")

        bdc = bt.get("dream_compile")
        cdc = ct.get("dream_compile")
        if bdc and cdc and bdc.get("total", 0) > 0 and cdc.get("total", 0) > 0:
            b_comp = bdc.get("compiled", 0)
            c_comp = cdc.get("compiled", 0)
            b_rate = bdc.get("rate_pct", 0.0)
            c_rate = cdc.get("rate_pct", 0.0)
            if c_comp < b_comp:
                record_regression(
                    f"dream-compile count regressed: {b_comp}/{bdc['total']} ({b_rate}%) -> "
                    f"{c_comp}/{cdc['total']} ({c_rate}%)",
                    area="dream_compile",
                )
            elif cdc["total"] <= bdc["total"] and c_rate + 1e-6 < b_rate:
                record_regression(
                    f"dream-compile share regressed: {b_rate}% -> {c_rate}%",
                    area="dream_compile",
                )
            elif c_comp > b_comp or c_rate > b_rate + 1e-6:
                improvements.append(
                    f"dream-compile improved: {b_comp}/{bdc['total']} ({b_rate}%) -> "
                    f"{c_comp}/{cdc['total']} ({c_rate}%)"
                )

        cgf = ct.get("gate_fast")
        if cgf and cgf.get("elapsed_s") is not None and cgf["elapsed_s"] > GATE_FAST_BUDGET_S:
            warnings.append(
                f"check.sh --fast wall time {cgf['elapsed_s']}s exceeded budget {GATE_FAST_BUDGET_S:.0f}s"
            )

        for key, label in (
            ("median_ratio", "median raw token ratio"),
            ("median_stripped_ratio", "median stripped token ratio"),
        ):
            bv = bt.get("tokens", {}).get(key)
            cv = ct["tokens"].get(key)
            if bv is not None and cv is not None and cv > bv + TOKEN_RATIO_TOLERANCE:
                record_regression(f"{label} regressed: {bv:.4f} -> {cv:.4f}", area="ports_or_codegen")

        if mode == "full":
            for key, label in (
                ("time_p50", "runtime p50"),
                ("time_p95", "runtime p95"),
                ("rss_p50", "peak RSS p50"),
                ("rss_p95", "peak RSS p95"),
            ):
                bv = bt.get("perf", {}).get(key)
                cv = ct["perf"].get(key)
                if bv is not None and cv is not None and cv > bv * PERF_REGRESSION_RATIO:
                    record_regression(f"{label} regressed >10%: {bv:.3f}x -> {cv:.3f}x", area="ports_or_codegen")
            bi_p50 = bt.get("interp", {}).get("ratio_p50")
            ci_p50 = ct.get("interp", {}).get("ratio_p50")
            if bi_p50 is not None and ci_p50 is not None:
                if ci_p50 > bi_p50 + 1e-6:
                    record_regression(f"interp ratio p50 regressed: {bi_p50:.2f}x -> {ci_p50:.2f}x", area="ports_or_codegen")
                elif ci_p50 < bi_p50 - 1e-6:
                    improvements.append(f"interp ratio p50 improved: {bi_p50:.2f}x -> {ci_p50:.2f}x")
            bi_over10 = bt.get("interp", {}).get("over_10x")
            ci_over10 = ct.get("interp", {}).get("over_10x")
            if bi_over10 is not None and ci_over10 is not None:
                if ci_over10 > bi_over10:
                    record_regression(f"interp ratio >10x count grew: {bi_over10} -> {ci_over10}", area="ports_or_codegen")
                elif ci_over10 < bi_over10:
                    improvements.append(f"interp ratio >10x count shrank: {bi_over10} -> {ci_over10}")
            bi_max = bt.get("interp", {}).get("ratio_max")
            ci_max = ct.get("interp", {}).get("ratio_max")
            if bi_max is not None and ci_max is not None:
                if ci_max > bi_max * PERF_REGRESSION_RATIO:
                    record_regression(f"interp ratio max regressed >10%: {bi_max:.2f}x -> {ci_max:.2f}x", area="ports_or_codegen")
                elif ci_max < bi_max / PERF_REGRESSION_RATIO:
                    improvements.append(f"interp ratio max improved: {bi_max:.2f}x -> {ci_max:.2f}x")

    return {
        "has_baseline": baseline is not None,
        "added_ports": added,
        "removed_ports": removed,
        "regressions": regressions,
        "improvements": improvements,
        "warnings": warnings,
    }


def merge_baseline_perf(port_records, baseline):
    """Preserve baseline perf/interp measurements when writing a --fast snapshot for unchanged ports."""
    if not baseline:
        return
    base_by_id = {r["id"]: r for r in baseline.get("ports", [])}
    for r in port_records:
        b = base_by_id.get(r["id"])
        if not b:
            continue
        if b.get("cp_sha") == r.get("cp_sha") and b.get("orig_sha") == r.get("orig_sha"):
            if r.get("perf") is None and b.get("perf") is not None:
                r["perf"] = b["perf"]
            if r.get("interp") is None and b.get("interp") is not None:
                r["interp"] = b["interp"]


def _port_id_from_relpath(relpath):
    """Extract active port id `group/name` from a repo-relative path under proto/ports/."""
    norm = relpath.replace("\\", "/").lstrip("./")
    if not norm.startswith("proto/ports/"):
        return None
    parts = norm.split("/")
    if len(parts) < 4:
        return None
    group = parts[2]
    if any(group.startswith(pfx) for pfx in RETIRED_DIR_PREFIXES):
        return None
    fname = parts[-1]
    if len(parts) == 4:
        if fname.endswith(".cp"):
            return f"{group}/{fname[:-3]}"
        if fname.endswith("_orig.cpp"):
            return f"{group}/{fname[:-9]}"
        if fname.endswith("_orig.cpp.strip"):
            return f"{group}/{fname[:-15]}"
        return None
    sub = parts[3]
    if sub in ("dream", "gen", "checkers", "interactors", "samples") and len(parts) == 5:
        stem = os.path.splitext(fname)[0]
        if sub == "samples":
            # samples/NAME.in, samples/NAME.1.in, etc.
            stem = stem.split(".")[0]
        return f"{group}/{stem}" if stem else None
    return None


def detect_touched_ports(repo_root=ROOT, base_ref="origin/main"):
    """Return the set of port IDs touched vs base_ref, or None if compiler/stdlib changed.

    When only ports/docs/reviews/frictions changed, returning the touched port IDs
    lets `--write` merge per port so a commit from an older checkout cannot clobber
    another commit's baseline entries (e.g. stale emitted_bytes on another port).
    """
    changed = set()
    try:
        ref_ok = subprocess.run(
            ["git", "rev-parse", "--verify", "--quiet", base_ref],
            cwd=repo_root,
            capture_output=True,
            text=True,
        ).returncode == 0
        diff_ref = base_ref if ref_ok else "HEAD"
        r = subprocess.run(
            ["git", "diff", "--name-only", diff_ref],
            cwd=repo_root,
            capture_output=True,
            text=True,
        )
        if r.returncode == 0:
            changed.update(line.strip() for line in r.stdout.splitlines() if line.strip())
        st = subprocess.run(
            ["git", "status", "--porcelain"],
            cwd=repo_root,
            capture_output=True,
            text=True,
        )
        if st.returncode == 0:
            for line in st.stdout.splitlines():
                if len(line) > 3:
                    p = line[3:].strip()
                    if " -> " in p:
                        p = p.split(" -> ", 1)[1]
                    if p:
                        changed.add(p)
    except OSError:
        return None

    for p in changed:
        norm = p.replace("\\", "/")
        if norm.startswith("crates/") or norm.startswith("stdlib/"):
            return None

    touched = set()
    for p in changed:
        pid = _port_id_from_relpath(p)
        if pid:
            touched.add(pid)
    return touched


def merge_baseline_ports(existing_baseline, new_report, touched_ids=None):
    """Merge new_report into existing_baseline per port.

    If touched_ids is a set/iterable of port IDs, ONLY those port entries are
    updated/inserted from new_report; every untouched port entry in
    existing_baseline is preserved verbatim so concurrent commits cannot race
    and overwrite each other's port metrics. Totals are recomputed over the
    merged port list.
    """
    if not existing_baseline:
        return new_report
    if touched_ids is None:
        return new_report

    touched = set(touched_ids)
    merged_by_id = {r["id"]: dict(r) for r in existing_baseline.get("ports", [])}
    new_by_id = {r["id"]: r for r in new_report.get("ports", [])}

    for pid in touched:
        if pid in new_by_id:
            rec = dict(new_by_id[pid])
            old = merged_by_id.get(pid)
            if old and rec.get("gate_mode") == "fast":
                if old.get("cp_sha") == rec.get("cp_sha") and old.get("orig_sha") == rec.get("orig_sha"):
                    if rec.get("perf") is None and old.get("perf") is not None:
                        rec["perf"] = old["perf"]
                    if rec.get("interp") is None and old.get("interp") is not None:
                        rec["interp"] = old["interp"]
            merged_by_id[pid] = rec
        else:
            merged_by_id.pop(pid, None)

    merged_ports = [merged_by_id[pid] for pid in sorted(merged_by_id)]
    nt = new_report.get("totals", {})
    et = existing_baseline.get("totals", {})
    merged_totals = compute_totals(
        merged_ports,
        new_report.get("friction_lint") or existing_baseline.get("friction_lint") or {"wa_trap": 0, "real_gap": 0, "stale_port": 0, "total_findings": 0},
        new_report.get("frictions") or existing_baseline.get("frictions") or {"files": 0, "total": 0, "open": 0, "closed": 0, "by_severity": {}, "by_goal": {}, "by_area": {}, "open_blockers": [], "open_highs": [], "warnings": []},
        bug_replay=nt.get("bug_replay") if nt.get("bug_replay") is not None else et.get("bug_replay"),
        revision=nt.get("revision") if nt.get("revision") is not None else et.get("revision"),
        spelling_drift=nt.get("spelling_drift") if nt.get("spelling_drift") is not None else et.get("spelling_drift"),
        dream_scores=nt.get("dream_scores") if nt.get("dream_scores") is not None else et.get("dream_scores"),
        dream_compile=nt.get("dream_compile") if nt.get("dream_compile") is not None else et.get("dream_compile"),
        gate_fast=nt.get("gate_fast") if nt.get("gate_fast") is not None else et.get("gate_fast"),
        taste_summary=nt.get("taste") if nt.get("taste") is not None else et.get("taste"),
    )

    out = dict(existing_baseline)
    out["schema_version"] = new_report.get("schema_version", existing_baseline.get("schema_version", 1))
    out["meta"] = new_report.get("meta", existing_baseline.get("meta"))
    out["totals"] = merged_totals
    out["frictions"] = new_report.get("frictions") or existing_baseline.get("frictions")
    out["friction_lint"] = new_report.get("friction_lint") or existing_baseline.get("friction_lint")
    if new_report.get("writable_from_docs") is not None:
        out["writable_from_docs"] = new_report["writable_from_docs"]
    if new_report.get("dreams") is not None:
        out["dreams"] = new_report["dreams"]
    out["ports"] = merged_ports
    return out


def ports_jsonl_path(baseline_path=DEFAULT_BASELINE):
    """Return the sorted line-per-port JSONL path paired with a baseline JSON file."""
    base, _ext = os.path.splitext(baseline_path)
    return base + ".ports.jsonl"


def load_baseline(baseline_path=DEFAULT_BASELINE):
    """Load baseline summary JSON + sibling sorted line-per-port `.ports.jsonl` (if present)."""
    if not baseline_path or not os.path.isfile(baseline_path):
        return None
    with open(baseline_path, encoding="utf-8") as f:
        data = json.load(f)
    ports_file = data.get("ports_file")
    if ports_file:
        jpath = os.path.join(os.path.dirname(os.path.abspath(baseline_path)), ports_file)
    else:
        jpath = ports_jsonl_path(baseline_path)
    if os.path.isfile(jpath):
        ports = []
        with open(jpath, encoding="utf-8") as pf:
            for line in pf:
                line = line.strip()
                if line:
                    ports.append(json.loads(line))
        data["ports"] = sorted(ports, key=lambda r: r.get("id", ""))
    return data


def write_baseline(report, out_path=DEFAULT_BASELINE, ports_only=False):
    """Write baseline in merge-safe split format: sorted 1-line-per-port JSONL + compact summary JSON."""
    os.makedirs(os.path.dirname(os.path.abspath(out_path)), exist_ok=True)
    jpath = ports_jsonl_path(out_path)
    ports_sorted = sorted(report.get("ports", []), key=lambda r: r.get("id", ""))
    with open(jpath, "w", encoding="utf-8") as pf:
        for rec in ports_sorted:
            pf.write(json.dumps(rec, sort_keys=True))
            pf.write("\n")
    if not ports_only:
        summary = {k: v for k, v in report.items() if k != "ports"}
        summary["ports_file"] = os.path.basename(jpath)
        with open(out_path, "w", encoding="utf-8") as f:
            json.dump(summary, f, indent=2, sort_keys=False)
            f.write("\n")


def update_baseline_ports(out_path, updated_records):
    """Update ONLY the named port lines in the sorted `.ports.jsonl` without touching summary JSON."""
    existing = load_baseline(out_path) or {"ports": []}
    by_id = {r["id"]: dict(r) for r in existing.get("ports", [])}
    for rec in updated_records:
        pid = rec["id"]
        new_rec = dict(rec)
        old = by_id.get(pid)
        if old and new_rec.get("gate_mode") == "fast":
            if old.get("cp_sha") == new_rec.get("cp_sha") and old.get("orig_sha") == new_rec.get("orig_sha"):
                if new_rec.get("perf") is None and old.get("perf") is not None:
                    new_rec["perf"] = old["perf"]
                if new_rec.get("interp") is None and old.get("interp") is not None:
                    new_rec["interp"] = old["interp"]
        by_id[pid] = new_rec
    merged_ports = [by_id[pid] for pid in sorted(by_id)]
    write_baseline({"ports": merged_ports}, out_path=out_path, ports_only=True)
    return merged_ports


def git_head_short():
    r = sh(["git", "rev-parse", "--short", "HEAD"], cwd=ROOT)
    return r.stdout.strip() if r.returncode == 0 else "unknown"


def print_frictions_report(fr, quiet=False):
    print(
        f"Frictions (research/frictions/*.md): {fr['files']} files, "
        f"{fr['open']} open ({fr['by_severity']['blocker']} blocker, "
        f"{fr['by_severity']['high']} high, {fr['by_severity']['medium']} medium, "
        f"{fr['by_severity']['low']} low), {fr['closed']} closed"
    )
    if fr["by_goal"]:
        goals_str = ", ".join(f"{g}={n}" for g, n in fr["by_goal"].items())
        print(f"  by goal: {goals_str}")
    if fr.get("by_area"):
        areas_str = ", ".join(f"{a}={n}" for a, n in fr["by_area"].items() if n)
        print(f"  by area: {areas_str}")
    if fr.get("by_area_queues"):
        print("  per-area stdlib queue:")
        for area in (*STDLIB_AREAS, "core"):
            q = fr["by_area_queues"].get(area)
            if q:
                print(f"    {area:12s} open={q['open']:3d}  high={q['high']:2d}  ({len(q['files'])} files)")
    if fr["open_blockers"]:
        print("  open blockers:")
        for b in fr["open_blockers"]:
            print(f"    - [{b['file']}] {b['title']}")
    if not quiet and fr["open_highs"]:
        print("  open highs:")
        for h in fr["open_highs"]:
            print(f"    - [{h['file']}] {h['title']}")
    if not quiet and fr["warnings"]:
        print(f"  schema warnings ({len(fr['warnings'])}):")
        for w in fr["warnings"][:10]:
            print(f"    ! {w}")


def dream_gaps():
    """scripts/dream-gaps.py, which a hyphen keeps from a plain import."""
    import importlib.util

    spec = importlib.util.spec_from_file_location(
        "dream_gaps", os.path.join(os.path.dirname(os.path.abspath(__file__)), "dream-gaps.py")
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def dreams_report(cpl):
    """What the port dreams need (plan/PORTING.md), without the per-file detail."""
    dg = dream_gaps()
    paths = dg.dreams()
    if not paths or not os.path.isfile(cpl):
        return None
    report = dg.report(cpl, paths)
    report.pop("files")
    return report


def print_summary(report, diff, show_ports=False, quiet=False):
    t = report["totals"]
    m = report["meta"]
    print(
        f"CP Lang Scoreboard ({m['mode']} mode, commit {m['commit']}, {m['elapsed_s']:.2f}s): "
        f"{t['ports_passed']}/{t['ports_total']} ports pass "
        f"({t.get('interactive_ports', 0)} interactive, {t.get('checker_ports', 0)} checker)"
    )
    tok = t["tokens"]
    print(
        f"  tokens:    cp={tok['cp_total']:,} vs orig={tok['orig_total']:,} "
        f"(median {tok['median_ratio']:.3f}x raw, {tok['median_stripped_ratio']:.3f}x stripped; "
        f">1.0x: {tok['over_1_0x_raw']} raw, {tok['over_1_0x_stripped']} stripped)"
        if tok["median_ratio"] is not None
        else "  tokens:    n/a"
    )
    dm = t.get("dream") or {}
    if dm.get("ports"):
        print(
            f"  dream:     {dm['ports']} ports, cp={dm['cp_total']:,} vs dream={dm['dream_total']:,} "
            f"(median {dm['median_ratio']:.3f}x, p90 {dm['p90_ratio']:.3f}x, "
            f"max {dm['max_ratio']:.3f}x {dm['max_port']})"
        )
    eb = t["emitted_bytes"]
    if eb["max"] is not None:
        print(
            f"  emitted:   median {eb['median']:,} B, p95 {eb['p95']:,} B, "
            f"max {eb['max']:,} B ({eb['max_port']}); >48KB: {eb['over_48kb']}, >64KB: {eb['over_64kb']}"
        )
    pf = t["perf"]
    if pf["measured"]:
        p50_tag = "PASS" if pf.get("p50_ok") else "FAIL"
        max_tag = "PASS" if pf.get("max_ok") else "FAIL"
        print(
            f"  perf:      {pf['measured']} measured ({pf['above_floor']} >= {pf['floor_ms']:.0f}ms), "
            f"time p50={pf['time_p50']}x (goal <={TIME_P50_GOAL}x: {p50_tag}) "
            f"p95={pf['time_p95']}x "
            f"max={pf['time_max']}x (goal <={TIME_MAX_GOAL}x: {max_tag}, {pf.get('over_1_5x', 0)} >{TIME_MAX_GOAL}x, {pf['time_max_port']}), "
            f"rss p50={pf['rss_p50']}x p95={pf['rss_p95']}x max={pf['rss_max']}x ({pf['rss_max_port']})"
        )
        if not quiet and m["mode"] == "full" and pf.get("worst_10"):
            print("  worst 10 runtime ratios:")
            for w in pf["worst_10"]:
                print(
                    f"    {w['id']:22} {w['time_ratio']:7.3f}x  "
                    f"cpp={w['cpp_ms']}ms orig={w['orig_ms']}ms  ({w['input_source']})"
                )
    ts = t.get("taste") or {}
    if ts.get("reviewed"):
        stale_n = t.get("taste_stale", ts.get("stale", 0))
        unrev_n = t.get("taste_unreviewed", ts.get("unreviewed", 0))
        pend_n = ts.get("pending", 0)
        hist = ts.get("histogram") or {}
        hist_str = " ".join(f"{k}:{hist.get(str(k), 0)}" for k in range(1, 6)) if hist else ""
        extra = f" [{hist_str}]" if hist_str else ""
        print(
            f"  taste:     {ts['reviewed']} reviewed, median {ts['median']}/5 "
            f"(min {ts['min']}, max {ts['max']}, <4: {ts['under_4']}, "
            f"pending: {pend_n}, stale: {stale_n}, unreviewed: {unrev_n}){extra}"
        )
    ip = t["interp"]
    if ip["measured"]:
        print(
            f"  interp:    {ip['passed']}/{ip['passed'] + ip['failed']} ok, "
            f"median {ip['median_ms']}ms, p95 {ip['p95_ms']}ms, max {ip['max_ms']}ms ({ip['max_port']})"
        )
    if ip.get("ratio_p50") is not None:
        bar_tag = "PASS" if ip.get("bar_ok") else "FAIL"
        ns_tag = "PASS" if ip.get("north_star_ok") else "FAIL"
        src_tag = " (cached)" if ip.get("cached") else ""
        print(
            f"  interp-ratio: p50={ip['ratio_p50']}x (bar <={INTERP_RATIO_BAR:.0f}x: {bar_tag}, "
            f"north star <={INTERP_RATIO_NORTH_STAR:.0f}x: {ns_tag}), "
            f"p95={ip['ratio_p95']}x, max={ip['ratio_max']}x ({ip['ratio_max_port']}), "
            f">10x: {ip.get('over_10x', 0)} ({ip['ratio_above_floor']} >= {INTERP_FLOOR_MS:.0f}ms){src_tag}"
        )
    tl = t.get("taste_lints")
    if tl:
        by = " ".join(f"{c}:{n}" for c, n in tl["by_lint"].items())
        print(f"  lints:     {tl['total']} taste findings in {tl['ports']} ports ({by})")
    fl = t["friction_lint"]
    print(f"  lint:      {fl['wa_trap']} wa_trap, {fl['real_gap']} real_gap, {fl['stale_port']} stale_port")
    br = t.get("bug_replay")
    if br:
        tag = "PASS" if br.get("bar_ok") else "FAIL"
        print(
            f"  bug-replay:{br['handled']}/{br['total']} handled ({br['handled_pct']}%, "
            f"bar >={br['bar_pct']:.0f}%: {tag}; {br['prevented']} prevented, "
            f"{br['caught_on_run']} caught on run, {br['left_to_diagnose']} left to diagnose)"
        )
        if br.get("by_class"):
            cls_parts = ", ".join(
                f"{cls}={d['handled']}/{d['total']}({d['handled_pct']:.0f}%)"
                for cls, d in br["by_class"].items()
            )
            print(f"    by class: {cls_parts}")
    rv = t.get("revision")
    if rv:
        parts = ", ".join(f"{c['id']}={c['lines_touched']}L/{c['hunks']}h" for c in rv["cases"])
        print(
            f"  revision:  {rv['passed']}/{rv['total']} ok, "
            f"{rv['total_lines_touched']} lines touched ({parts})"
        )
    sd = t.get("spelling_drift")
    if sd:
        cparts = []
        for cname, cent in (sd.get("constructs") or {}).items():
            cparts.append(
                f"{cname}={cent['non_canonical']} (active {cent['active_non_canonical']}, canon {cent['canonical']})"
            )
        print(
            f"  drift:     {sd['total_non_canonical']} non-canonical "
            f"({sd['active_non_canonical']} in active/dream files): " + "; ".join(cparts)
        )
    ds = t.get("dream_scores")
    if ds and ds.get("scored"):
        print(
            f"  dreams:    {ds['scored']} scored, median {ds['median']}/5, "
            f"{ds['under_4']} under 4 (range {ds['min']}..{ds['max']})"
        )
    dc = t.get("dream_compile")
    if dc and dc.get("total"):
        print(
            f"  dream-ok:  {dc['compiled']}/{dc['total']} ({dc['rate_pct']}%) check with zero errors"
        )
    gf = t.get("gate_fast")
    if gf and gf.get("elapsed_s") is not None:
        tag = "PASS" if gf.get("ok") else "FAIL"
        print(f"  gate-fast: {gf['elapsed_s']}s (budget <={gf['budget_s']:.0f}s: {tag})")
    wfd = report.get("writable_from_docs")
    if wfd and wfd.get("latest"):
        cur = wfd["latest"]
        print(
            f"  docs-wfd:  {cur['compiled']}/{cur['total']} compile ({cur['compile_rate']:.1%}, "
            f"floor {wfd['min_compile_rate']:.1%}), {cur['passed']}/{cur['total']} pass "
            f"({cur['pass_rate']:.1%}, floor {wfd['min_pass_rate']:.1%}) [{cur['model']}]"
        )
    print_frictions_report(report["frictions"], quiet=quiet)
    if report.get("dreams"):
        for line in dream_gaps().summary_lines(report["dreams"], top=5 if quiet else 10):
            print("  " + line)

    if diff["has_baseline"]:
        print(
            f"  baseline:  +{len(diff['added_ports'])} added, -{len(diff['removed_ports'])} removed, "
            f"{len(diff['regressions'])} regressions, {len(diff['improvements'])} improvements"
        )
        if diff["added_ports"]:
            sample = ", ".join(diff["added_ports"][:8])
            more = f" (+{len(diff['added_ports']) - 8} more)" if len(diff["added_ports"]) > 8 else ""
            print(f"    added: {sample}{more}")
    if diff["regressions"]:
        print("  REGRESSIONS:")
        for r in diff["regressions"]:
            print(f"    FAIL: {r}")
    for w in diff.get("warnings", []):
        print(f"    WARN: {w}")

    if show_ports:
        print("\nPer-port details:")
        for r in report["ports"]:
            perf_s = ""
            if (r.get("perf") or {}).get("measured"):
                p = r["perf"]
                rss_s = f"{p['rss_ratio']}x" if p.get("rss_ratio") is not None else "n/a"
                perf_s = f" | t={p['cpp_ms']}ms/{p['orig_ms']}ms ({p['time_ratio']}x) rss={rss_s}"
            interp_s = ""
            if (r.get("interp") or {}).get("ms") is not None:
                interp_s = f" | interp={r['interp']['ms']}ms"
            dream_s = ""
            if r.get("dream_tokens") is not None:
                dream_s = f" | dream={r['dream_tokens']} ({r['dream_ratio']:.2f}x)"
            raw_s = f"{r['token_ratio']:.2f}x" if r.get("token_ratio") is not None else "n/a"
            strip_s = f"{r['stripped_token_ratio']:.2f}x" if r.get("stripped_token_ratio") is not None else "n/a"
            print(
                f"  [{r['gate']:4}] {r['id']:22} tok={r['cp_tokens']:4}/{r['orig_tokens']:5} "
                f"({raw_s}, strip {strip_s}) "
                f"cpp={r['emitted_bytes'] or 0:6}B{dream_s}{perf_s}{interp_s}"
            )


def build_report(args):
    t0 = time.perf_counter()
    only = set(x.strip() for x in args.ports.split(",") if x.strip()) if args.ports else None
    port_dirs = [x.strip() for x in args.dirs.split(",") if x.strip()] if args.dirs else None
    ports = discover_ports(port_dirs=port_dirs, only=only)
    if only and not ports:
        sys.exit(f"scoreboard: no ports matched --ports {args.ports}")

    cpl = find_cpl_binary(args.cpl)
    if not os.path.isfile(cpl):
        sys.exit(f"scoreboard: cpl binary not found at {cpl}; run `cargo build --release -p cpl-cli`")

    mode = "fast" if args.fast else "full"
    jobs = args.jobs or max(1, min(8, os.cpu_count() or 4))

    fast_map = run_batch_fast_checks(ports, cpl, jobs=jobs)
    with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as ex:
        tok_list = list(ex.map(measure_tokens, ports))

    full_map = {}
    if mode == "full":
        explicit_specs = load_explicit_specs()
        with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as ex:
            futs = {
                ex.submit(
                    evaluate_full_port,
                    p,
                    fast_map[p["id"]],
                    cpl,
                    explicit_specs,
                    args.runs,
                    args.cases,
                    args.interp_timeout,
                    args.san,
                ): p["id"]
                for p in ports
            }
            for fut in concurrent.futures.as_completed(futs):
                full_map[futs[fut]] = fut.result()

    port_records = []
    for p, tok in zip(ports, tok_list):
        fr = fast_map[p["id"]]
        eb = fr["emitted_bytes"]
        fast_pass = (
            fr["check_ok"]
            and fr["fmt_ok"]
            and fr["lint_warnings"] == 0
            and fr["lint_errors"] == 0
            and fr["build_ok"]
            and (eb is not None and eb <= SIZE_LIMIT)
        )
        detail_parts = []
        if not fr["check_ok"]:
            detail_parts.append(f"check: {fr['check_err'] or 'failed'}")
        if not fr["fmt_ok"]:
            detail_parts.append("unformatted")
        if fr["lint_warnings"] or fr["lint_errors"]:
            detail_parts.append(f"lint: {fr['lint_warnings']}w/{fr['lint_errors']}e")
        if not fr["build_ok"]:
            detail_parts.append(f"build: {fr['build_err'] or 'failed'}")
        elif eb is not None and eb > SIZE_LIMIT:
            detail_parts.append(f"over 64KB ({eb}B)")

        rec = {
            "id": p["id"],
            "path": p["path"],
            "orig_path": p["orig_path"],
            "gate": "pass" if fast_pass else "fail",
            "gate_mode": mode,
            "gate_detail": "; ".join(detail_parts) if detail_parts else "ok",
            "check_ok": fr["check_ok"],
            "fmt_ok": fr["fmt_ok"],
            "lint_warnings": fr["lint_warnings"],
            "taste_lints": fr["taste_lints"],
            "taste_lints_by_lint": fr["taste_lints_by_lint"],
            "build_ok": fr["build_ok"],
            "emitted_bytes": eb,
            "over_48kb": bool(eb is not None and eb > SIZE_WARN),
            "over_64kb": bool(eb is not None and eb > SIZE_LIMIT),
            **tok,
            "has_gen": p["gen"] is not None,
            "has_samples": bool(p["samples"]),
            "interactive": p["interactor"] is not None,
            "has_checker": p["checker"] is not None,
            "perf": None,
            "interp": None,
        }

        if mode == "full":
            full = full_map[p["id"]]
            rec["perf"] = full["perf"]
            rec["interp"] = full["interp"]
            if not full["diff_ok"]:
                rec["gate"] = "fail"
                rec["gate_detail"] = full["diff_detail"]
            elif fast_pass:
                rec["gate_detail"] = full["diff_detail"]

        port_records.append(rec)

    fl_summary = run_friction_lint(ports)
    fr_summary = summarize_frictions(args.frictions_dir)
    bug_replay = run_bug_replay_suite(cpl) if not only else None
    revision = run_revision_suite(cpl) if not only else None
    spelling_drift = measure_spelling_drift() if not only else None
    dream_scores = summarize_dream_scores() if not only else None
    dreams_rep = dreams_report(cpl) if not only else None
    dream_compile = summarize_dream_compile(dreams_rep) if dreams_rep else None

    baseline = load_baseline(args.baseline) if args.baseline else None

    gate_fast = load_gate_fast_time(baseline=baseline) if not only else None

    if mode == "fast" and args.write and baseline:
        merge_baseline_perf(port_records, baseline)

    taste_summary = summarize_taste_reviews(port_records) if not only else None

    totals = compute_totals(
        port_records,
        fl_summary,
        fr_summary,
        bug_replay=bug_replay,
        revision=revision,
        spelling_drift=spelling_drift,
        dream_scores=dream_scores,
        dream_compile=dream_compile,
        gate_fast=gate_fast,
        taste_summary=taste_summary,
    )
    if mode == "fast" and not only and baseline and totals["interp"].get("ratio_p50") is None:
        base_ip = (baseline.get("totals") or {}).get("interp") or {}
        if base_ip.get("ratio_p50") is not None:
            cached_ip = dict(base_ip)
            cached_ip["cached"] = True
            totals["interp"] = cached_ip
    wfd_path = os.path.join(ROOT, "research", "writable-from-docs.json")
    wfd_data = None
    if os.path.isfile(wfd_path):
        with open(wfd_path, "r", encoding="utf-8") as f:
            wfd_data = json.load(f)

    report = {
        "schema_version": 1,
        "meta": {
            "commit": git_head_short(),
            "mode": mode,
            "platform": platform.platform(),
            "cores": os.cpu_count() or 1,
            "elapsed_s": round(time.perf_counter() - t0, 2),
        },
        "totals": totals,
        "frictions": fr_summary,
        "friction_lint": fl_summary,
        "writable_from_docs": wfd_data,
        "dreams": dreams_rep,
        "ports": port_records,
    }
    scope = detect_change_scope() if not only else None
    diff = diff_against_baseline(report, baseline, mode=mode, subset=bool(only), scope=scope)
    if wfd_data and wfd_data.get("latest"):
        cur = wfd_data["latest"]
        if cur["compile_rate"] + 1e-9 < wfd_data.get("min_compile_rate", 0.0):
            diff["regressions"].append(
                f"writable-from-docs compile_rate {cur['compile_rate']:.1%} < floor {wfd_data['min_compile_rate']:.1%}"
            )
        if cur["pass_rate"] + 1e-9 < wfd_data.get("min_pass_rate", 0.0):
            diff["regressions"].append(
                f"writable-from-docs pass_rate {cur['pass_rate']:.1%} < floor {wfd_data['min_pass_rate']:.1%}"
            )
    return report, diff


def main(argv=None):
    ap = argparse.ArgumentParser(description="CP Lang validation scoreboard and friction summary")
    ap.add_argument("--fast", action="store_true", help="cheap gate mode for scripts/check.sh (no C++ compile/run)")
    ap.add_argument("--full", action="store_true", help="full mode with C++ -O2 perf, RSS and interpreter timing (default)")
    ap.add_argument("--san", action="store_true", help="also compile and run ASan/UBSan in full mode")
    ap.add_argument("--ports", help="comma-separated list of port names or group/name IDs")
    ap.add_argument("--dirs", help="comma-separated port directories to scan")
    ap.add_argument("--jobs", type=int, default=0, help="parallel workers")
    ap.add_argument("--runs", type=int, default=3, help="timing runs per binary in full mode (default 3)")
    ap.add_argument("--cases", type=int, default=5, help="random diff cases per port in full mode (default 5)")
    ap.add_argument("--interp-timeout", type=float, default=10.0, help="interpreter timeout per case in seconds")
    ap.add_argument("--baseline", default=DEFAULT_BASELINE, help="baseline scoreboard JSON for diffing")
    ap.add_argument("--no-baseline", action="store_true", help="skip baseline comparison")
    ap.add_argument("--check", action="store_true", help="exit 1 on any regression or hard gate failure")
    ap.add_argument("--write", action="store_true", help="write scoreboard baseline to --out (merges per port into sorted .ports.jsonl)")
    ap.add_argument("--update", nargs="+", metavar="PORT", help="update only the named port entries in .ports.jsonl without touching summary JSON")
    ap.add_argument("--all-ports", action="store_true", help="with --write, rewrite all port entries instead of merging only touched ports")
    ap.add_argument("--out", default=DEFAULT_BASELINE, help="output JSON path for --write")
    ap.add_argument("--json", action="store_true", help="print JSON report to stdout")
    ap.add_argument("--show-ports", action="store_true", help="print per-port table")
    ap.add_argument("--quiet", action="store_true", help="omit verbose open-high and schema-warning lists")
    ap.add_argument("--frictions", action="store_true", help="print friction summary over research/frictions/*.md and exit")
    ap.add_argument("--frictions-dir", default=FRICTIONS_DIR, help="directory containing friction markdown reports")
    ap.add_argument("--cpl", help="path to cpl binary")
    args = ap.parse_args(argv)

    if args.no_baseline:
        args.baseline = None

    if args.update:
        args.ports = ",".join(args.update)

    if args.frictions:
        fr = summarize_frictions(args.frictions_dir)
        if args.json:
            print(json.dumps(fr, indent=2))
        else:
            print_frictions_report(fr)
        if args.check and fr["open_blockers"]:
            return 1
        return 0

    report, diff = build_report(args)
    if report.get("dreams") is None and not args.update:
        report["dreams"] = dreams_report(find_cpl_binary(args.cpl))
    if args.update:
        update_baseline_ports(args.out, report.get("ports", []))
    elif args.write:
        to_write = report
        if not args.all_ports and os.path.isfile(args.out):
            existing_out = load_baseline(args.out)
            if args.ports:
                touched_ids = {r["id"] for r in report.get("ports", [])}
            else:
                touched_ids = detect_touched_ports()
            if touched_ids is not None:
                to_write = merge_baseline_ports(existing_out, report, touched_ids=touched_ids)
        write_baseline(to_write, out_path=args.out)

    if args.json:
        print(json.dumps({"report": report, "diff": diff}, indent=2))
    else:
        print_summary(report, diff, show_ports=args.show_ports, quiet=args.quiet)

    if args.check and diff["regressions"]:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
