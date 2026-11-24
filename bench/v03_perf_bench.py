#!/usr/bin/env python3
"""Cached, matched-algorithm v0.3 C++ codegen performance benchmark harness.

Measures the runtime ratio (emitted C++ / matched C++ original), each side's
fastest user+sys CPU time over its runs, across:
  1. corpus   -- corpus/problems/*/{port.cp,orig.cpp} (max-N via corpus_generators, gen.py --max, or tests/*.in)
  2. dryrun   -- research/dryrun/v03/p*.cp at large N vs bench/dryrun-matched/p*.cpp
                 (the same algorithm as research/dryrun/p*_ref.cpp with the fast
                 I/O hand-written contest C++ uses), else the _ref.cpp itself
  3. zero-cost -- bench/zero-cost/* via `cpl bench --zero-cost`

The emitted C++ is compiled with `g++ -std=c++17 -O2 -w`, originals with
`-std=gnu++20` (judge originals use C++20 headers). An original g++ cannot build
(C++23: deducing `this`, <format>) is built with `--cxx23 CMD` (a clang; default
`python3 -m ziglang c++` when the running interpreter has `pip install ziglang`) at `-std=c++23`, and its pair's emitted C++ with the same
CMD, so both sides of a ratio use one compiler. Outputs are verified for exact
token-level output agreement (1e-6 float tolerance), or by the problem's
`checker.py` where it has one (problems with many valid answers). Also reported: peak RSS
of each run (emitted / original) and g++ time on the emitted C++ (taken while
`--jobs` builds run at once, so compare runs made with the same `--jobs`).
Every pair that does not reach a ratio is listed with its reason.

Each program may keep a tuned in-language variant next to it (`perf.cp`
beside a corpus `port.cp`, as `cpl bench --corpus` reads it, else `<stem>.perf.cp`,
else a dry run's `<stem>_w.cp`): widths, `unchecked:`,
layout, no inline C++ beyond the documented escape. Its tuned ratio is the
better of the two; the summary prints the tuned max against the 1.1x bar
(AGENTS.md goal 3) and the outliers left above it. Compiled C++ originals and
generated inputs are cached by content hash so reruns after codegen changes
only recompile CP Lang programs.
"""
import argparse
import concurrent.futures
import glob
import hashlib
import importlib.util
import json
import math
import os
import re
import resource
import shutil
import signal
import subprocess
import sys
import threading
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(ROOT, "bench"))

CPL = os.path.join(ROOT, "target", "release", "cpl")
CLEAN_ENV = {
    k: v
    for k, v in os.environ.items()
    if k not in ("CPL_SPEC", "CPL_STD", "CPL_EDITION")
}

# Dry-run problems whose _ref.cpp uses the same asymptotic algorithm as v03/.
DRYRUN_MATCHED = [
    ("p01_ordinary", "p01_ordinary_gen.py", None),
    ("p02_movies", "p02_movies_gen.py", None),
    ("p03_forest", "p03_forest_gen.py", None),
    ("p04_factory", "p04_factory_gen.py", None),
    ("p06_labyrinth", "p06_labyrinth_gen.py", None),
    ("p07_dijkstra", "p07_dijkstra_gen.py", None),
    ("p08_roads", "p08_roads_gen.py", None),
    ("p09_topo", "p09_topo_gen.py", None),
    ("p10_matching", "p10_matching_gen.py", None),
    ("p14_elevator", "p14_elevator_gen.py", None),
    ("p20_point", "p20_point_gen.py", None),
]


def big_stack():
    try:
        resource.setrlimit(resource.RLIMIT_STACK, (1 << 30, 1 << 30))
    except OSError:
        pass


def limit_child(timeout):
    """A big stack, and a CPU limit so the child dies on its own even when
    this harness is killed first: an orphan spinning for hours skews every
    later run on the machine."""
    big_stack()
    secs = int(timeout) + 5
    try:
        resource.setrlimit(resource.RLIMIT_CPU, (secs, secs + 5))
    except (OSError, ValueError):
        pass


def sha256_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()[:16]


import os as _os, sys as _sys
_sys.path.insert(0, _os.path.join(_os.path.dirname(_os.path.abspath(__file__)), "..", "corpus", "lib"))
from judge_tokens import same_tokens  # noqa: E402  (cpl test's own comparison)

def outputs_agree(cp, inp, out, ans):
    """Exact tokens, or, where the problem ships a `checker.py` (many valid
    answers), the checker's verdict on `out` against the original's `ans`."""
    if same_tokens(open(ans, errors="replace").read(), open(out, errors="replace").read()):
        return True
    checker = os.path.join(os.path.dirname(cp), "checker.py")
    if not os.path.exists(checker):
        return False
    r = subprocess.run([sys.executable, checker, inp, out, ans], capture_output=True, timeout=300)
    return r.returncode == 0


# A child forked from this process starts with its resident set, so its own
# `ru_maxrss` never reads below this interpreter's; `time` forks the program
# from a small process and reports the program's peak alone.
TIME = shutil.which("time", path="/usr/bin:/bin")


def run_once(binary, inp, outp, timeout):
    rss_file = outp + ".rss"
    cmd = [TIME, "-f", "%M", "-o", rss_file, binary] if TIME else [binary]
    with open(inp, "rb") as fi, open(outp, "wb") as fo:
        pr = subprocess.Popen(
            cmd,
            stdin=fi,
            stdout=fo,
            stderr=subprocess.DEVNULL,
            preexec_fn=lambda: limit_child(timeout),
            start_new_session=True,
        )
        killer = threading.Timer(timeout, lambda: os.killpg(pr.pid, signal.SIGKILL))
        killer.start()
        _, status, usage = os.wait4(pr.pid, 0)
        pr.returncode = os.waitstatus_to_exitcode(status)
        timed_out = not killer.is_alive()
        killer.cancel()
    if timed_out:
        return timeout, "timeout", 0
    # Parallel jobs and other load inflate wall time, not CPU time.
    cpu = usage.ru_utime + usage.ru_stime
    rss = usage.ru_maxrss
    if TIME:
        with open(rss_file) as f:
            rss = int(f.read().split()[-1])
    return cpu, "ok" if pr.returncode == 0 else f"exit {pr.returncode}", rss


CXX23_SHIM = os.path.join(ROOT, "bench", "cxx23-shim")


# Originals that use C++23 (deducing `this`, <format>) need a newer compiler
# than the system g++; the ziglang wheel ships one clang on every platform.
DEFAULT_CXX23 = f"{sys.executable} -m ziglang c++" if importlib.util.find_spec("ziglang") else ""


def timed_gxx(src, out, std="c++17", cxx=None):
    """g++ (or the `cxx` command line) on `src`; returns (CompletedProcess, ms)."""
    cmd = (cxx or ["g++"]) + [f"-std={std}", "-O2", "-w"] + (["-I", CXX23_SHIM] if cxx else [])
    t0 = time.perf_counter()
    r = subprocess.run(cmd + [src, "-o", out], capture_output=True, text=True)
    ms = (time.perf_counter() - t0) * 1000.0
    return r, ms


def compile_ms(bin_path):
    try:
        return float(open(bin_path + ".ms").read())
    except (OSError, ValueError):
        return None


def compile_cpp_cached(src, cache_dir, tag, std="gnu++20", cxx=None):
    key = f"{tag}_{'cxx23_' if cxx else ''}{std}_{sha256_file(src)}"
    bin_path = os.path.join(cache_dir, key)
    if os.path.exists(bin_path):
        return bin_path, None
    tmp = bin_path + f".tmp.{os.getpid()}.{threading.get_ident()}"
    r, ms = timed_gxx(src, tmp, std, cxx)
    if r.returncode != 0:
        if os.path.exists(tmp):
            os.remove(tmp)
        lines = r.stderr.strip().splitlines() or ["g++ failed"]
        err = next((l for l in lines if "error" in l), lines[0]).replace(ROOT + "/", "")[:160]
        return None, err
    open(bin_path + ".ms", "w").write(f"{ms:.1f}")
    os.replace(tmp, bin_path)
    return bin_path, None


def build_cpl_cached(cp_src, cache_dir, cpl_hash):
    key = f"cpl_{cpl_hash}_{sha256_file(cp_src)}"
    cpp_path = os.path.join(cache_dir, key + ".cpp")
    bin_path = os.path.join(cache_dir, key)
    if os.path.exists(bin_path) and os.path.exists(cpp_path):
        return bin_path, cpp_path, None
    tmp_cpp = cpp_path + f".tmp.{os.getpid()}.{threading.get_ident()}"
    r = subprocess.run(
        [CPL, "build", cp_src, "-o", tmp_cpp],
        capture_output=True,
        text=True,
        env=CLEAN_ENV,
    )
    if r.returncode != 0:
        if os.path.exists(tmp_cpp):
            os.remove(tmp_cpp)
        err = ((r.stderr or r.stdout).strip().splitlines() or ["cpl build failed"])[0][:160]
        return None, None, f"cpl build: {err}"
    os.replace(tmp_cpp, cpp_path)
    tmp_bin = bin_path + f".tmp.{os.getpid()}.{threading.get_ident()}"
    g, ms = timed_gxx(cpp_path, tmp_bin)
    if g.returncode != 0:
        if os.path.exists(tmp_bin):
            os.remove(tmp_bin)
        err = (g.stderr.strip().splitlines() or ["g++ failed"])[0][:160]
        return None, cpp_path, f"g++ emitted: {err}"
    open(bin_path + ".ms", "w").write(f"{ms:.1f}")
    os.replace(tmp_bin, bin_path)
    return bin_path, cpp_path, None


def gen_dryrun_input(name, cache_dir):
    """Generate a large deterministic input for matched dry-run problems."""
    inp = os.path.join(cache_dir, f"dryrun_in_{name}.txt")
    if os.path.exists(inp) and os.path.getsize(inp) > 0:
        return inp
    import random

    rng = random.Random(42)
    lines = []
    if name == "p01_ordinary":
        t = 200000
        lines.append(str(t))
        for _ in range(t):
            lines.append(str(rng.randint(1, 10**9)))
    elif name == "p02_movies":
        n = 200000
        lines.append(str(n))
        for _ in range(n):
            a = rng.randint(1, 10**9 - 1000)
            b = a + rng.randint(1, 1000)
            lines.append(f"{a} {b}")
    elif name == "p03_forest":
        n, q = 1000, 200000
        lines.append(f"{n} {q}")
        for _ in range(n):
            lines.append("".join("*" if rng.random() < 0.3 else "." for _ in range(n)))
        for _ in range(q):
            y1, y2 = sorted((rng.randint(1, n), rng.randint(1, n)))
            x1, x2 = sorted((rng.randint(1, n), rng.randint(1, n)))
            lines.append(f"{y1} {x1} {y2} {x2}")
    elif name == "p04_factory":
        n, t = 200000, 10**9
        lines.append(f"{n} {t}")
        lines.append(" ".join(str(rng.randint(1, 10**9)) for _ in range(n)))
    elif name == "p06_labyrinth":
        n, m = 800, 800
        lines.append(f"{n} {m}")
        grid = [["." for _ in range(m)] for _ in range(n)]
        for r in range(n):
            for c in range(m):
                if (r, c) not in ((0, 0), (n - 1, m - 1)) and rng.random() < 0.22:
                    grid[r][c] = "#"
        grid[0][0] = "A"
        grid[n - 1][m - 1] = "B"
        for row in grid:
            lines.append("".join(row))
    elif name == "p07_dijkstra":
        n, m = 100000, 200000
        lines.append(f"{n} {m}")
        for i in range(2, n + 1):
            lines.append(f"{rng.randint(1, i - 1)} {i} {rng.randint(1, 10**6)}")
        for _ in range(m - (n - 1)):
            lines.append(f"{rng.randint(1, n)} {rng.randint(1, n)} {rng.randint(1, 10**6)}")
    elif name == "p08_roads":
        n, m = 100000, 200000
        lines.append(f"{n} {m}")
        for _ in range(m):
            u = rng.randint(1, n)
            v = rng.randint(1, n)
            lines.append(f"{u} {v}")
    elif name == "p09_topo":
        n, m = 100000, 200000
        lines.append(f"{n} {m}")
        for _ in range(m):
            u = rng.randint(1, n - 1)
            v = rng.randint(u + 1, n)
            lines.append(f"{u} {v}")
    elif name == "p10_matching":
        n = 200000
        lines.append(str(n))
        for i in range(2, n + 1):
            lines.append(f"{rng.randint(1, i - 1)} {i}")
    elif name == "p14_elevator":
        n, x = 20, 10**9
        lines.append(f"{n} {x}")
        lines.append(" ".join(str(rng.randint(1, x)) for _ in range(n)))
    elif name == "p20_point":
        t = 200000
        lines.append(str(t))
        for _ in range(t):
            vals = [str(rng.randint(-(10**9), 10**9)) for _ in range(6)]
            lines.append(" ".join(vals))
    with open(inp, "w") as f:
        f.write("\n".join(lines) + "\n")
    return inp


def derived_inputs(cp, cache_dir):
    """The worst cases `cpl bench` derives from `cp`'s `read()` types, captured by
    passing a program that saves its stdin as `--fast`. `cpl bench` types only
    top-level reads; a port that reads inside a loop or fn gets none."""
    if re.search(r"^[ \t]+.*\bread\(", open(cp).read(), re.M):
        return []
    cap = os.path.join(cache_dir, f"derived_{sha256_file(cp)}")
    if not os.path.isdir(cap):
        tmp = cap + f".tmp.{os.getpid()}.{threading.get_ident()}"
        os.makedirs(tmp)
        saver = os.path.join(tmp, "save.sh")
        with open(saver, "w") as f:
            f.write('#!/bin/sh\nexec cat > "$CAPDIR/$(date +%s%N).in"\n')
        os.chmod(saver, 0o755)
        subprocess.run(
            [CPL, "bench", cp, "--fast", saver, "--format", "json"],
            capture_output=True,
            env={**CLEAN_ENV, "CAPDIR": tmp},
            timeout=300,
        )
        os.remove(saver)
        os.replace(tmp, cap)
    return sorted(os.path.join(cap, f) for f in os.listdir(cap) if os.path.getsize(os.path.join(cap, f)) > 0)


def scaled_gen_inputs(gen, cache_dir):
    """`gen.py <seed> <size>` at sizes 10, 100, ... up to the first that fails, times
    out or writes over 16 MB; the two largest, since the largest may break a
    constraint the generator does not bound (both programs must agree on one)."""
    outs = []
    for size in (10, 100, 1000, 10_000, 100_000):
        out = os.path.join(cache_dir, f"gen_{sha256_file(gen)}_{size}.in")
        if not os.path.exists(out):
            tmp = out + f".tmp.{os.getpid()}.{threading.get_ident()}"
            try:
                with open(tmp, "wb") as fo:
                    ok = subprocess.run([sys.executable, gen, "1", str(size)], stdout=fo,
                                        stderr=subprocess.DEVNULL, timeout=20).returncode == 0
            except subprocess.TimeoutExpired:
                ok = False
            if not ok or os.path.getsize(tmp) > 16 << 20:
                os.remove(tmp)
                open(out, "w").close()
            else:
                os.replace(tmp, out)
        if os.path.getsize(out) == 0:
            break
        outs.append(out)
    return outs[-2:]


def load_unmatched():
    """Pairs listed in bench/unmatched.txt: id -> why port and orig differ in algorithm."""
    out = {}
    with open(os.path.join(ROOT, "bench", "unmatched.txt")) as f:
        for line in f:
            line = line.strip()
            if line and not line.startswith("#"):
                pid, _, why = line.partition(" ")
                out[pid] = why.strip()
    return out


def collect_items(suites, only, cache_dir):
    items = []
    if "dryrun" in suites:
        for name, _, _ in DRYRUN_MATCHED:
            if only and name not in only:
                continue
            cp = os.path.join(ROOT, "research", "dryrun", "v03", f"{name}.cp")
            orig = os.path.join(ROOT, "bench", "dryrun-matched", f"{name}.cpp")
            if not os.path.exists(orig):
                orig = os.path.join(ROOT, "research", "dryrun", f"{name}_ref.cpp")
            if os.path.exists(cp) and os.path.exists(orig):
                items.append(("dryrun", name, cp, orig, lambda n=name: [gen_dryrun_input(n, cache_dir)]))

    if "corpus" in suites:
        cg_by_prob = {}
        try:
            import corpus_generators as cg

            for s in cg.SPECS:
                cg_by_prob[os.path.basename(s[0])] = s
        except Exception:
            pass
        for orig in sorted(glob.glob(os.path.join(ROOT, "corpus", "problems", "*", "orig.cpp"))):
            d = os.path.dirname(orig)
            pid = os.path.basename(d)
            if only and pid not in only:
                continue
            cp = os.path.join(d, "port.cp")
            if not os.path.exists(cp):
                continue

            def make_input(p=pid, prob_dir=d, spec=cg_by_prob.get(pid)):
                if spec is not None:
                    out = os.path.join(cache_dir, f"corpus_gen_{p}.in")
                    if not (os.path.exists(out) and os.path.getsize(out) > 0):
                        try:
                            spec[4](out, True, {})
                        except Exception:
                            if os.path.exists(out):
                                os.remove(out)
                    if os.path.exists(out) and os.path.getsize(out) > 0:
                        return [out]
                gen = os.path.join(prob_dir, "gen.py")
                if os.path.exists(gen) and "--max" in open(gen).read():
                    out = os.path.join(cache_dir, f"corpus_max_{p}.in")
                    if not (os.path.exists(out) and os.path.getsize(out) > 0):
                        with open(out, "wb") as fo:
                            if subprocess.run([sys.executable, gen, "--max"], stdout=fo, timeout=300).returncode:
                                fo.truncate(0)
                    if os.path.getsize(out) > 0:
                        return [out]
                if os.path.exists(os.path.join(prob_dir, "interactor.py")):
                    return []
                ins = sorted(glob.glob(os.path.join(prob_dir, "tests", "*.in")))
                # A problem's own gen.py knows its alphabet and invariants; inputs
                # derived from `read()` types alone can break them (random letters
                # for a string over "ab?"), so they stand in only without one.
                if os.path.exists(gen):
                    derived, scaled = [], scaled_gen_inputs(gen, cache_dir)
                else:
                    derived, scaled = derived_inputs(os.path.join(prob_dir, "port.cp"), cache_dir), []
                return derived + scaled + (
                    [max(ins, key=os.path.getsize)] if ins else []
                )

            items.append(("corpus", pid, cp, orig, make_input))

    if "bench" in suites:
        empty_in = os.path.join(cache_dir, "empty.in")
        if not os.path.exists(empty_in):
            open(empty_in, "w").close()
        for sub in ("containers", "flow", "math", "ranges", "seq", "strings"):
            for cp in sorted(glob.glob(os.path.join(ROOT, "bench", sub, "*.cp"))):
                if cp.endswith(".perf.cp"):
                    continue
                stem = os.path.basename(cp)[:-3]
                name = f"{sub}/{stem}"
                if only and name not in only and stem not in only:
                    continue
                orig = cp[:-3] + ".cpp"
                if not os.path.exists(orig):
                    continue
                items.append(("bench", name, cp, orig, lambda p=empty_in: [p]))
    return items


TUNED_BAR = 1.1


def tuned_path(cp):
    """The tuned in-language variant kept next to `cp`, if there is one."""
    d, base = os.path.split(cp)
    t = os.path.join(d, "perf.cp") if base == "port.cp" else cp[:-3] + ".perf.cp"
    if not os.path.exists(t):
        # The dry runs keep their hand-rooted in-language rewrite as `<stem>_w.cp`.
        t = cp[:-3] + "_w.cp"
    return t if os.path.exists(t) else None


def percentile(sorted_vals, q):
    if not sorted_vals:
        return 0.0
    k = (len(sorted_vals) - 1) * q
    lo = math.floor(k)
    hi = math.ceil(k)
    if lo == hi:
        return sorted_vals[lo]
    return sorted_vals[lo] * (hi - k) + sorted_vals[hi] * (k - lo)


def summarize(rows, min_ms):
    measured = [r for r in rows if r.get("status") == "ok" and max(r["port_ms"], r["orig_ms"]) >= min_ms]
    ratios = sorted(r["ratio"] for r in measured)
    # An unmatched pair's tuned variant may be the original's algorithm, written in the
    # language: it counts toward the tuned bar, though the idiomatic port does not.
    rewritten = [
        r for r in rows
        if r.get("status") == "unmatched" and r.get("tuned_ratio") and max(r["port_ms"], r["orig_ms"]) >= min_ms
    ]
    tuned = sorted(
        [min(r["ratio"], r.get("tuned_ratio") or r["ratio"]) for r in measured]
        + [r["tuned_ratio"] for r in rewritten]
    )
    return {
        "total_ok": sum(1 for r in rows if r.get("status") == "ok"),
        "measured_ge_min_ms": len(measured),
        "p50": round(percentile(ratios, 0.50), 3),
        "p90": round(percentile(ratios, 0.90), 3),
        "p95": round(percentile(ratios, 0.95), 3),
        "max": round(ratios[-1], 3) if ratios else 0.0,
        "gt_1_1": sum(1 for x in ratios if x > 1.1),
        "gt_1_5": sum(1 for x in ratios if x > 1.5),
        "tuned_max": round(tuned[-1], 3) if tuned else 0.0,
        "tuned_gt_1_1": sum(1 for x in tuned if x > TUNED_BAR),
        "tuned_variants": sum(1 for r in measured if r.get("tuned_ratio")) + len(rewritten),
        **resource_summary([r for r in rows if r.get("status") == "ok"]),
    }


def print_tuned(st):
    print(
        f"  tuned max={st['tuned_max']:.3f}x (bar <= {TUNED_BAR}x; over it: {st['tuned_gt_1_1']}; "
        f"tuned variants timed: {st['tuned_variants']})"
    )


def resource_summary(ok):
    rss = sorted(r["rss_ratio"] for r in ok if r.get("rss_ratio"))
    rss_kb = sorted(r["port_rss_kb"] for r in ok if r.get("port_rss_kb"))
    cms = sorted(r["compile_ms"] for r in ok if r.get("compile_ms") is not None)
    return {
        "rss_ratio_p50": round(percentile(rss, 0.50), 3),
        "rss_ratio_max": round(rss[-1], 3) if rss else 0.0,
        "rss_max_mb": round(rss_kb[-1] / 1024, 1) if rss_kb else 0.0,
        "compile_ms_p50": round(percentile(cms, 0.50)),
        "compile_ms_max": round(cms[-1]) if cms else 0,
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--suite", default="dryrun,corpus,bench", help="comma-separated: dryrun,corpus,bench")
    ap.add_argument("--only", default="", help="comma-separated problem names")
    ap.add_argument("--runs", type=int, default=5)
    ap.add_argument("--timeout", type=float, default=10.0)
    ap.add_argument("--min-ms", type=float, default=10.0)
    ap.add_argument("--jobs", type=int, default=min(8, os.cpu_count() or 4))
    ap.add_argument("--cache", default="/tmp/cpl-perf-cache")
    ap.add_argument("--json-out", default="")
    ap.add_argument(
        "--cxx23",
        default=os.environ.get("CPL_BENCH_CXX23", DEFAULT_CXX23),
        help="C++23 compiler command for originals g++ cannot build (default: the ziglang"
        " package's clang when installed, `pip install ziglang`)",
    )
    args = ap.parse_args()
    cxx23 = args.cxx23.split() or None

    os.makedirs(args.cache, exist_ok=True)
    cpl_hash = sha256_file(CPL)
    suites = {s.strip() for s in args.suite.split(",") if s.strip()}
    only = {s.strip() for s in args.only.split(",") if s.strip()}
    items = collect_items(suites, only, args.cache)

    def build_one(item):
        suite, name, cp, orig, input_fn = item
        pbin, cpp_path, perr = build_cpl_cached(cp, args.cache, cpl_hash)
        if perr:
            return {"suite": suite, "name": name, "status": "skip", "reason": perr}
        tag = f"orig_{name.replace('/', '__')}"
        obin, oerr = compile_cpp_cached(orig, args.cache, tag)
        cxx = None
        if oerr and cxx23:
            obin, cerr = compile_cpp_cached(orig, args.cache, tag, "c++23", cxx23)
            if cerr:
                return {"suite": suite, "name": name, "status": "skip", "reason": f"g++ and --cxx23 orig: {cerr}"}
            gxx_pbin, pbin = pbin, None
            pbin, perr = compile_cpp_cached(cpp_path, args.cache, "emitted", "c++17", cxx23)
            if perr:
                return {"suite": suite, "name": name, "status": "skip", "reason": f"--cxx23 emitted: {perr}"}
            if os.path.exists(gxx_pbin + ".ms"):
                open(pbin + ".ms", "w").write(open(gxx_pbin + ".ms").read())
            cxx, oerr = args.cxx23, None
        if oerr:
            return {"suite": suite, "name": name, "status": "skip", "reason": f"g++ orig: {oerr}"}
        tbin, terr = None, None
        tuned = tuned_path(cp)
        if tuned:
            tbin, tcpp, terr = build_cpl_cached(tuned, args.cache, cpl_hash)
            if not terr and cxx:
                tbin, terr = compile_cpp_cached(tcpp, args.cache, "tuned", "c++17", cxx23)
        inp = input_fn()
        if not inp:
            if os.path.exists(os.path.join(os.path.dirname(cp), "interactor.py")):
                return {"suite": suite, "name": name, "status": "skip", "reason": "interactive: needs the interactor, which the bench does not run"}
            return {"suite": suite, "name": name, "status": "skip", "reason": "no input: no generator, no tests, and `cpl bench` derives none"}
        return {
            "suite": suite,
            "name": name,
            "pbin": pbin,
            "obin": obin,
            "cp": cp,
            "cpp_path": cpp_path,
            "inp": inp,
            "cxx": cxx,
            "tbin": tbin,
            "tuned_error": terr,
            "status": "built",
        }

    built = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=args.jobs) as ex:
        for res in ex.map(build_one, items):
            built.append(res)

    rows = []
    unmatched = load_unmatched()
    # Per process: two benches sharing a cache must not compare each other's output.
    out_p = os.path.join(args.cache, f"out_p.{os.getpid()}.txt")
    out_o = os.path.join(args.cache, f"out_o.{os.getpid()}.txt")
    for b in built:
        if b["status"] != "built":
            rows.append(b)
            continue
        # Among the candidate inputs both programs agree on, time the one the original is slowest on.
        agreed, tried = [], []
        for inp in b["inp"]:
            t_o, s_o, rss_o = run_once(b["obin"], inp, out_o, args.timeout)
            if s_o != "ok":
                tried.append(f"orig {s_o}")
                continue
            t_p, s_p, rss_p = run_once(b["pbin"], inp, out_p, args.timeout)
            if s_p != "ok":
                tried.append(f"port {s_p}")
                continue
            if not outputs_agree(b["cp"], inp, out_p, out_o):
                tried.append("outputs differ")
                continue
            agreed.append((t_o, t_p, rss_o, rss_p, inp))
        if not agreed:
            status = "mismatch" if "outputs differ" in tried else "skip"
            reason = ", ".join(sorted(set(tried)))
            rows.append({**b, "status": status, "reason": f"{len(b['inp'])} input(s): {reason}"})
            continue
        t_o, t_p, rss_o, rss_p, inp = max(agreed)
        b["inp"] = inp
        ts_o = [t_o]
        ts_p = [t_p]
        if max(t_o, t_p) * 1000.0 >= args.min_ms:
            for _ in range(args.runs - 1):
                to, _, _ = run_once(b["obin"], b["inp"], out_o, args.timeout)
                tp, _, _ = run_once(b["pbin"], b["inp"], out_p, args.timeout)
                ts_o.append(to)
                ts_p.append(tp)
        med_o = min(ts_o) * 1000.0
        med_p = min(ts_p) * 1000.0
        ratio = med_p / med_o if med_o > 0 else 1.0
        tuned = {}
        if b.get("tbin"):
            ts_t = []
            for _ in range(len(ts_p)):
                tt, s_t, _ = run_once(b["tbin"], b["inp"], out_p, args.timeout)
                if s_t != "ok" or not outputs_agree(b["cp"], b["inp"], out_p, out_o):
                    ts_t = []
                    tuned = {"tuned_error": f"tuned {s_t if s_t != 'ok' else 'output differs'} on {b['inp']}"}
                    break
                ts_t.append(tt)
            if ts_t and med_o > 0:
                tuned = {"tuned_ms": round(min(ts_t) * 1000.0, 2), "tuned_ratio": round(min(ts_t) * 1000.0 / med_o, 3)}
        elif b.get("tuned_error"):
            tuned = {"tuned_error": b["tuned_error"]}
        why = unmatched.get(b["name"]) if b["suite"] == "corpus" else None
        rows.append(
            {
                "suite": b["suite"],
                "name": b["name"],
                "status": "unmatched" if why else "ok",
                **({"reason": why} if why else {}),
                "port_ms": round(med_p, 2),
                "orig_ms": round(med_o, 2),
                "ratio": round(ratio, 3),
                "port_rss_kb": rss_p,
                "orig_rss_kb": rss_o,
                "rss_ratio": round(rss_p / rss_o, 3) if rss_o else None,
                "compile_ms": compile_ms(b["pbin"]),
                "orig_compile_ms": compile_ms(b["obin"]),
                "cpp_path": b["cpp_path"],
                **tuned,
                **({"cxx": b["cxx"]} if b["cxx"] else {}),
            }
        )

    for s in sorted(suites):
        sub = [r for r in rows if r["suite"] == s]
        st = summarize(sub, args.min_ms)
        print(f"=== {s} (n={st['measured_ge_min_ms']} >= {args.min_ms}ms, ok={st['total_ok']}/{len(sub)}) ===")
        print(
            f"  p50={st['p50']:.3f}x  p90={st['p90']:.3f}x  p95={st['p95']:.3f}x  max={st['max']:.3f}x  "
            f"(>1.1x: {st['gt_1_1']}, >1.5x: {st['gt_1_5']}); tuned max={st['tuned_max']:.3f}x (bar <= {TUNED_BAR}x)"
        )
        measured = [r for r in sub if r.get("status") == "ok" and max(r["port_ms"], r["orig_ms"]) >= args.min_ms]
        print(
            f"  peak RSS emitted/orig p50={st['rss_ratio_p50']:.3f}x max={st['rss_ratio_max']:.3f}x "
            f"(largest {st['rss_max_mb']} MB); g++ on emitted C++ p50={st['compile_ms_p50']} ms max={st['compile_ms_max']} ms"
        )
        print_tuned(st)
        for r in sorted(measured, key=lambda x: -x["ratio"])[:20]:
            t = f"  tuned={r['tuned_ratio']:6.3f}x" if r.get("tuned_ratio") else ""
            print(f"  {r['name']:18s}  port={r['port_ms']:7.2f}ms  orig={r['orig_ms']:7.2f}ms  ratio={r['ratio']:6.3f}x{t}")
        failed = [r for r in sub if r.get("status") != "ok"]
        if failed:
            print(f"  not measured: {len(failed)}")
            for r in sorted(failed, key=lambda x: x["name"]):
                print(f"    {r['name']:18s}  {r['status']}: {r.get('reason', '')}")

    overall = summarize(rows, args.min_ms)
    print(f"=== OVERALL (n={overall['measured_ge_min_ms']} >= {args.min_ms}ms) ===")
    print(
        f"  p50={overall['p50']:.3f}x  p90={overall['p90']:.3f}x  p95={overall['p95']:.3f}x  max={overall['max']:.3f}x  "
        f"(>1.1x: {overall['gt_1_1']}, >1.5x: {overall['gt_1_5']}); tuned max={overall['tuned_max']:.3f}x (bar <= {TUNED_BAR}x)"
    )
    print_tuned(overall)
    measured = [r for r in rows if r.get("status") == "ok" and max(r["port_ms"], r["orig_ms"]) >= args.min_ms]
    left = [r for r in measured if min(r["ratio"], r.get("tuned_ratio") or r["ratio"]) > TUNED_BAR]
    for r in sorted(left, key=lambda x: -min(x["ratio"], x.get("tuned_ratio") or x["ratio"])):
        if r.get("tuned_ratio"):
            why = f"tuned {r['tuned_ratio']:.3f}x"
        else:
            why = r.get("tuned_error") or "no tuned variant"
        print(f"    over {TUNED_BAR}x tuned: {r['name']:18s}  idiomatic {r['ratio']:.3f}x, {why}")

    if args.json_out:
        with open(args.json_out, "w") as f:
            json.dump({"cpl_hash": cpl_hash, "overall": overall, "rows": rows}, f, indent=2)


if __name__ == "__main__":
    main()
