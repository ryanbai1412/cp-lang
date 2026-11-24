#!/usr/bin/env python3
"""Corpus-wide max-N benchmark of `cpl build` against the original C++.

The P0 target this tracks: the p95 runtime ratio of emitted C++17 over the
accepted original, both built with `g++ -std=c++17 -O2`, stays under 1.1x
across 200+ corpus ports, with every emitted file under 64 KB.

    cargo build --release -p cpl-cli
    python3 bench/corpus_p95_bench.py                  # full sweep, rewrites the report (about 15 minutes)
    python3 bench/corpus_p95_bench.py --ports dpl,noi22p5 --no-write
    python3 bench/corpus_p95_bench.py --check          # exit 1 unless p95 < 1.1x over 200+ ports and all files < 64 KB

One port is measured per problem. The candidates come from
research/perf/corpus-500-registry.json (refresh it with bench/corpus_registry.py), the
canonical port first. A port is measured when

  1. `cpl build` accepts it and g++ compiles both it and its original,
  2. a maximum-size input exists for it, and
  3. on that input both programs exit 0 and print the same tokens.

Inputs come from two places. The hand-written judge-limit generators of
bench/rust_max_n_audit.py are used where one exists. Every other port gets the
worst cases `cpl bench` derives from its `read()` types (all sizes at their
maximum; values random, all smallest, all largest, ascending, descending,
alternating; edge lists as a path and a star), captured by passing a program
that saves its stdin as `--fast`. `cpl bench` only types top-level reads,
so a port that reads inside a loop or function is skipped until it has a
hand-written generator: its derived input would be truncated. A judge-limit input is timed when both
programs agree on it; otherwise the derived input the original takes longest
on, among those they agree on.

Timing: `--runs` runs of each binary, alternating, with an unlimited stack;
the ratio is median wall time over median wall time. Peak RSS is a separate
run under /usr/bin/time. Nothing tracked is written except the two outputs:

    research/perf/corpus-p95-results.json   every candidate, measured or not, and why
    research/perf/corpus-p95-report.md      percentiles and the top-20 tail offenders
"""
import argparse, concurrent.futures, json, math, os, platform, re, resource, shutil, signal, statistics, subprocess, sys, tempfile, threading, time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BENCH = os.path.join(ROOT, "bench")
RESULTS = os.path.join(ROOT, "research", "perf")
CPL = os.environ.get("CPL") or os.path.join(ROOT, "target", "release", "cpl")
REGISTRY = os.path.join(RESULTS, "corpus-500-registry.json")
OUT_JSON = os.path.join(RESULTS, "corpus-p95-results.json")
OUT_MD = os.path.join(RESULTS, "corpus-p95-report.md")
SIZE_LIMIT = 64 * 1024
TARGET_P95, TARGET_PORTS = 1.1, 200
FLOOR_S = 0.010          # below this on both sides a ratio is process start-up noise
FORM_RANK = {"regression": 0, "runnable": 1, "spec": 2, "draft": 3}
SHAPES = ("random", "all smallest", "all largest", "ascending", "descending", "alternating", "path", "star")


def sh(cmd, **kw):
    return subprocess.run(cmd, capture_output=True, text=True, **kw)


def big_stack():
    resource.setrlimit(resource.RLIMIT_STACK, (resource.RLIM_INFINITY, resource.RLIM_INFINITY))


def run(binary, inp, outp, timeout, rss=False):
    """Run once on `inp`; return (wall seconds, peak RSS KB or 0, status)."""
    cmd = ["/usr/bin/time", "-f", "%M", "-o", outp + ".rss", binary] if rss else [binary]
    with open(inp) as fi, open(outp, "w") as fo:
        t0 = time.perf_counter()
        pr = subprocess.Popen(cmd, stdin=fi, stdout=fo, stderr=subprocess.DEVNULL, preexec_fn=big_stack, start_new_session=True)
        killer = threading.Timer(timeout, lambda: os.killpg(pr.pid, signal.SIGKILL))
        killer.start()
        pr.wait()
        wall = time.perf_counter() - t0
        timed_out = not killer.is_alive(); killer.cancel()
    if timed_out:
        return timeout, 0, "timeout"
    kb = 0
    if rss:
        try: kb = int(open(outp + ".rss").read().split()[-1])
        except (OSError, ValueError, IndexError): pass
    return wall, kb, "ok" if pr.returncode == 0 else f"exit {pr.returncode}"


import os as _os, sys as _sys
_sys.path.insert(0, _os.path.join(_os.path.dirname(_os.path.abspath(__file__)), "..", "corpus", "lib"))
from judge_tokens import same_tokens  # noqa: E402  (cpl test's own comparison)

def hand_generators():
    """Return (by_port_rel, by_problem, audit2) from rust_max_n_audit and corpus_generators."""
    by_rel, by_prob, audit2 = {}, {}, None
    try:
        import corpus_generators as cg
        for s in cg.SPECS:
            by_rel[s[0]] = s
            by_prob.setdefault(os.path.basename(s[0]), s)
    except Exception as e:
        print(f"note: expanded corpus generators unavailable ({e!r})", file=sys.stderr)
    try:
        import rust_max_n_audit as audit2
        for s in audit2.SPECS:
            by_rel[s[0]] = s
            by_prob[os.path.basename(s[0])] = s
    except Exception as e:                       # the sweep still runs on `cpl bench` inputs alone
        print(f"note: hand-written generators unavailable ({e!r})", file=sys.stderr)
    return by_rel, by_prob, audit2


def candidates(only):
    reg = json.load(open(REGISTRY))
    canonical = {name: p["canonical"] for name, p in reg["problems"].items()}
    by_problem = {}
    for p in reg["ports"]:
        if p["form"] == "support" or p.get("original_language") != "cpp" or not p.get("original"): continue
        if not (os.path.exists(os.path.join(ROOT, p["path"])) and os.path.exists(os.path.join(ROOT, p["original"]))): continue
        stem = os.path.basename(p["path"])[:-3]
        if only and p["problem"] not in only and stem not in only: continue
        by_problem.setdefault(p["problem"], []).append(p)
    if only:
        found = set(by_problem) | {os.path.basename(p["path"])[:-3] for ports in by_problem.values() for p in ports}
        if only - found:
            sys.exit(f"--ports: no measurable port for {', '.join(sorted(only - found))} (see research/perf/corpus-500-registry.json)")
    for name, ports in by_problem.items():
        ports.sort(key=lambda p: (p["path"] != canonical.get(name), FORM_RANK.get(p["form"], 9), p["path"]))
    return by_problem


def prepare(port, work, timeout, hand, hand_by_prob, audit2):
    """Build both sides and find the inputs they agree on. Returns the candidate's record."""
    rel = port["path"]
    r = {"problem": port["problem"], "port": rel, "original": port["original"], "form": port["form"]}
    d = os.path.join(work, rel.removeprefix("corpus/problems/").removeprefix("proto/ports/").removesuffix(".cp").replace("/", "__")); os.makedirs(d)
    r["dir"] = d
    name = os.path.basename(rel)[:-3]
    src = os.path.join(d, name + ".cp"); shutil.copy(os.path.join(ROOT, rel), src)
    cpp = os.path.join(d, "port.cpp")
    b = sh([CPL, "build", src, "-o", cpp])
    if b.returncode:
        r["skip"] = "cpl build: " + ((b.stderr or b.stdout).strip().splitlines() or ["failed"])[0].replace(d + "/", "")[:160]; return r
    r["emitted_bytes"] = os.path.getsize(cpp)

    orig = os.path.join(ROOT, port["original"])
    spec = hand.get(rel.removeprefix("corpus/problems/").removeprefix("proto/ports/").removesuffix(".cp")) or hand_by_prob.get(port["problem"])
    osrc = orig
    if spec and audit2 and name in audit2.bc5.DRIVERS:      # grader-style originals need a stdin driver
        if name == "noiwc22p3":
            with open(os.path.join(d, "word.h"), "w") as f: f.write(audit2.bc5.WORD_H)
        osrc = os.path.join(d, "driver.cpp")
        with open(osrc, "w") as f: f.write(audit2.bc5.DRIVERS[name].replace("{orig}", spec[1]))
    for tag, s in (("port", cpp), ("orig", osrc)):
        c = sh(["g++", "-std=c++17", "-O2", "-w", "-I", d, s, "-o", os.path.join(d, tag)])
        if c.returncode:
            r["skip"] = f"g++ on the {'emitted C++' if tag == 'port' else 'original'}: " + re.sub(r"/tmp/\w+\.o", "a.o", (c.stderr.strip().splitlines() or ["failed"])[0].replace(d + "/", "").replace(ROOT + "/", ""))[-160:]; return r

    inputs = []                                   # (label, path, compare)
    if spec:
        path = os.path.join(d, "hand.in")
        def run_orig(p):
            run(os.path.join(d, "orig"), p, p + ".gen", timeout); return open(p + ".gen").read()
        try:
            spec[4](path, True, {"run_orig": run_orig}); inputs.append((f"judge limits ({spec[3]})", path, spec[5]))
        except Exception as e:
            r["hand_generator_error"] = repr(e)[:160]
    # `cpl bench` types top-level reads only, so a port that reads inside a loop or function
    # would get a truncated input that a scanf-style original still answers.
    if not inputs:
        if re.search(r"^[ \t]+.*\bread\(", open(src).read(), re.M):
            r["skip"] = "needs a hand-written generator: reads inside a loop or function, which `cpl bench` cannot type"; return r
        cap = os.path.join(d, "cap"); os.makedirs(cap)
        saver = os.path.join(d, "save.sh")
        with open(saver, "w") as f: f.write('#!/bin/sh\nexec cat > "$CAPDIR/$(date +%s%N).in"\n')
        os.chmod(saver, 0o755)
        cb = sh([CPL, "bench", src, "--fast", saver, "--format", "json"], env={**os.environ, "CAPDIR": cap})
        try:
            shapes = [x["shape"] for x in json.loads(cb.stdout)["runs"]]
        except (ValueError, KeyError):
            shapes = []
            r["cpl_bench"] = ((cb.stderr or cb.stdout).strip().splitlines() or ["no output"])[0].replace(d + "/", "")[:160]
        files = sorted(os.listdir(cap))
        if len(files) == len(shapes):
            inputs += [(f"cpl bench: {shape}", os.path.join(cap, f), "exact") for shape, f in zip(shapes, files)]
    if not inputs:
        r["skip"] = "no maximum-size input: " + r.get("cpl_bench", "nothing captured"); return r

    agreed, tried = [], []
    for i, (label, path, mode) in enumerate(inputs):
        o, p = os.path.join(d, f"{i}.orig"), os.path.join(d, f"{i}.port")
        wo, _, so = run(os.path.join(d, "orig"), path, o, timeout)
        if so != "ok": tried.append(f"{label}: original {so}"); continue
        wp, _, sp = run(os.path.join(d, "port"), path, p, timeout)
        if sp != "ok": tried.append(f"{label}: port {sp}"); continue
        a, b2 = open(o).read(), open(p).read()
        ok = audit2.same(mode, a, b2) if mode != "exact" else (bool(a.strip()) and same_tokens(a, b2))
        if not ok: tried.append(f"{label}: outputs differ"); continue
        agreed.append((wo, label, path))
    r["inputs_tried"] = len(inputs); r["inputs_agreed"] = len(agreed)
    if tried: r["rejected_inputs"] = tried
    if not agreed:
        r["skip"] = "no input both programs agree on: " + "; ".join(tried)[:300]; return r
    judge = [x for x in agreed if x[1].startswith("judge limits")]   # real limits beat derived ones
    _, r["input"], r["input_path"] = judge[0] if judge else max(agreed)
    r["input_bytes"] = os.path.getsize(r["input_path"])
    return r


def measure(r, runs, timeout):
    d, inp = r["dir"], r["input_path"]
    walls = {"orig": [], "port": []}
    for _ in range(runs):
        for tag in ("orig", "port"):
            w, _, st = run(os.path.join(d, tag), inp, os.path.join(d, "t." + tag), timeout)
            if st != "ok": r["skip"] = f"{tag} {st} while timing"; return
            walls[tag].append(w)
    for tag in ("orig", "port"):
        r[f"{tag}_s"] = round(statistics.median(walls[tag]), 5)
        r[f"{tag}_rss_kb"] = run(os.path.join(d, tag), inp, os.path.join(d, "t." + tag), timeout, rss=True)[1]
    r["time_ratio"] = round(r["port_s"] / r["orig_s"], 3)
    r["rss_ratio"] = round(r["port_rss_kb"] / max(r["orig_rss_kb"], 1), 3)
    r["above_floor"] = max(r["orig_s"], r["port_s"]) >= FLOOR_S


def pct(xs, q):
    """Nearest-rank percentile: the smallest value with at least q% of the data at or below it."""
    xs = sorted(xs)
    return xs[max(0, math.ceil(q / 100 * len(xs)) - 1)]


def stats(xs):
    return {"n": len(xs), "median": round(statistics.median(xs), 3), "p90": pct(xs, 90), "p95": pct(xs, 95), "p99": pct(xs, 99), "worst": max(xs)}


def summarise(rows, built_sizes):
    done = [r for r in rows if "time_ratio" in r]
    heavy = [r for r in done if r["above_floor"]]
    s = {"measured": len(done), "above_floor": len(heavy), "floor_ms": FLOOR_S * 1e3,
         "max_emitted_bytes": max(built_sizes.values(), default=0), "emitted_files": len(built_sizes),
         "emitted_over_64kb": sorted(p for p, b in built_sizes.items() if b >= SIZE_LIMIT)}
    if done:
        s["time_all"] = stats([r["time_ratio"] for r in done]); s["rss_all"] = stats([r["rss_ratio"] for r in done])
        s["largest_emitted_port"] = max(built_sizes, key=built_sizes.get)
    if heavy:
        s["time_above_floor"] = stats([r["time_ratio"] for r in heavy]); s["rss_above_floor"] = stats([r["rss_ratio"] for r in heavy])
    s["target_met"] = bool(heavy) and len(done) >= TARGET_PORTS and s["time_above_floor"]["p95"] < TARGET_P95 and not s["emitted_over_64kb"]
    return s


def line(st):
    return f"median {st['median']:.2f}x, p90 {st['p90']:.2f}x, p95 {st['p95']:.2f}x, p99 {st['p99']:.2f}x, worst {st['worst']:.2f}x"


def offender(i, r):
    return (f"{i}. `{r['port'].removeprefix('corpus/problems/').removeprefix('proto/ports/')}`: **{r['time_ratio']:.2f}x** ({r['orig_s'] * 1e3:.0f} ms -> {r['port_s'] * 1e3:.0f} ms), "
            f"RSS {r['rss_ratio']:.2f}x ({r['orig_rss_kb'] / 1024:.0f} -> {r['port_rss_kb'] / 1024:.0f} MB), "
            f"{r['emitted_bytes'] / 1024:.1f} KB emitted, input: {r['input']}")


def report(meta, s, rows):
    done = [r for r in rows if "time_ratio" in r]
    heavy = sorted((r for r in done if r["above_floor"]), key=lambda r: -r["time_ratio"])
    skipped = [r for r in rows if "skip" in r]
    out = ["# Corpus max-N benchmark: emitted C++17 against the original", "",
           "Generated by `python3 bench/corpus_p95_bench.py`; do not edit by hand. Every row is in [`corpus-p95-results.json`](corpus-p95-results.json).", "",
           f"Run at `{meta['commit']}` with `cpl` {meta['cpl_version']} (language 0.2.2), {meta['gxx']}, `-std=c++17 -O2` on both sides, "
           f"{meta['runs']} timed runs per binary, {meta['cpu']} ({meta['cores']} cores).", "",
           "## Headline", "",
           f"The target is a p95 runtime ratio under {TARGET_P95}x across {TARGET_PORTS}+ ports with every emitted file under 64 KB. "
           f"**{'Met' if s['target_met'] else 'Not met'}.**", ""]
    if done:
        out += [f"- **{s['measured']} ports measured**, one per problem, each on a maximum-size input where its output matches the original's.",
                f"- **Runtime ratio over the {s['above_floor']} ports where either side runs {s['floor_ms']:.0f} ms or more:** {line(s['time_above_floor'])}." if heavy else "- No port ran long enough to time.",
                f"- Runtime ratio over all {s['measured']}: {line(s['time_all'])}. Below {s['floor_ms']:.0f} ms a ratio is mostly process start-up, so the line above is the one to quote.",
                f"- **Peak RSS ratio** over all {s['measured']}: {line(s['rss_all'])}." + (f" Over the {s['above_floor']} timed ones: {line(s['rss_above_floor'])}." if heavy else ""),
                f"- **Largest emitted C++17 file: {s['max_emitted_bytes'] / 1024:.1f} KB** (`{s['largest_emitted_port'].removeprefix('corpus/problems/').removeprefix('proto/ports/')}`), over all {s['emitted_files']} port files `cpl build` accepts; "
                + (f"{len(s['emitted_over_64kb'])} at or over 64 KB." if s["emitted_over_64kb"] else "none reaches 64 KB."), ""]
        over = [r for r in heavy if r["time_ratio"] >= TARGET_P95]
        out += [f"{len(over)} of the {len(heavy)} timed ports are at or over {TARGET_P95}x; for p95 to pass, at most {len(heavy) - math.ceil(0.95 * len(heavy))} may be.", "",
                "## Top 20 runtime offenders", "", "Ranked by ratio among ports where either side runs at least " + f"{s['floor_ms']:.0f} ms.", ""]
        out += [offender(i, r) for i, r in enumerate(heavy[:20], 1)]
        out += ["", "## Top 10 peak-RSS offenders", ""]
        out += [f"{i}. `{r['port'].removeprefix('corpus/problems/').removeprefix('proto/ports/')}`: **{r['rss_ratio']:.2f}x** ({r['orig_rss_kb'] / 1024:.1f} -> {r['port_rss_kb'] / 1024:.1f} MB), runtime {r['time_ratio']:.2f}x"
                for i, r in enumerate(sorted(done, key=lambda r: -r["rss_ratio"])[:10], 1)]
        hand = sum(r["input"].startswith("judge limits") for r in done)
        out += ["", "## What was measured", "",
                f"{hand} ports ran on the hand-written judge-limit inputs of `rust_max_n_audit.py` and `corpus_generators.py`; the other {len(done) - hand} ran on the slowest worst case `cpl bench` derives from the port's `read()` types "
                "that both programs answer identically. Those inputs put every size at its declared maximum, but they are not judge data: a generated case can miss a problem's structure "
                "(a tree, a permutation), and such cases are only kept when the original still exits 0 with the same output as the port. "
                f"The original's median time is at least 100 ms on {sum(r['orig_s'] >= 0.1 for r in done)} ports and at least 1 s on {sum(r['orig_s'] >= 1 for r in done)}."]
    # A problem is blocked by whatever stopped the candidate that got furthest.
    stage = ["cpl build", "g++ on the emitted C++", "g++ on the original", "needs a hand-written generator", "no maximum-size input"]
    measured, furthest = {r["problem"] for r in done}, {}
    for r in skipped:
        key = r["skip"].split(":")[0]
        rank = stage.index(key) if key in stage else len(stage)
        if r["problem"] not in measured and rank >= furthest.get(r["problem"], (-1, ""))[0]:
            furthest[r["problem"]] = (rank, key)
    reasons = {}
    for _, key in furthest.values():
        reasons[key] = reasons.get(key, 0) + 1
    out += ["", "## What was left out", "",
            f"{meta['problems']} problems have a C++ original and a port; {len(furthest)} could not be measured. "
            "A problem is measured through its first candidate port that passes, and counted here under what stopped the candidate that got furthest:", ""]
    out += [f"- {n} problems: {k}" for k, n in sorted(reasons.items(), key=lambda kv: -kv[1])]
    if len(done) < TARGET_PORTS:
        out += ["", f"{meta['problems'] - reasons.get('cpl build', 0)} problems have a port `cpl build` accepts, so {TARGET_PORTS} measured ports is within reach of inputs alone. "
                "A port under `needs a hand-written generator` reads inside a loop or function, where `cpl bench` has no types to derive a maximum-size input from: "
                "give it a generator in `SPECS` in `corpus_generators.py` or `rust_max_n_audit.py` and the next sweep measures it."]
    out += ["", "The reason for each file is its `skip` field in the JSON.", ""]
    triage_path = os.path.join(RESULTS, "triage-analysis.md")
    if len(done) >= 50 and os.path.exists(triage_path):
        with open(triage_path) as tf:
            out += ["", tf.read().rstrip(), ""]
    return "\n".join(out)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--ports", help="comma-separated problem names or port file names; implies nothing about --no-write")
    ap.add_argument("--tail", type=int, metavar="N", help="measure the N slowest problems of the committed results (adds to --ports)")
    ap.add_argument("--runs", type=int, default=5, help="timed runs per binary (default 5)")
    ap.add_argument("--jobs", type=int, default=max(1, (os.cpu_count() or 2) // 2), help="parallel builds; timing is always serial")
    ap.add_argument("--timeout", type=float, default=20, help="seconds per run (default 20)")
    ap.add_argument("--no-write", action="store_true", help="print the summary without rewriting the committed outputs")
    ap.add_argument("--check", action="store_true", help="exit 1 unless the target is met")
    ap.add_argument("--keep", help="keep work files in this directory")
    a = ap.parse_args()
    if not os.path.exists(CPL): sys.exit(f"{CPL} not found: cargo build --release -p cpl-cli")
    sys.path.insert(0, BENCH)
    hand, hand_by_prob, audit2 = hand_generators()
    only = set(a.ports.split(",")) if a.ports else set()
    if a.tail:
        past = [r for r in json.load(open(OUT_JSON))["ports"] if r.get("time_ratio")]
        only |= {r["problem"] for r in sorted(past, key=lambda r: -r["time_ratio"])[:a.tail]}
    by_problem = candidates(only or None)
    work = a.keep or tempfile.mkdtemp(prefix="cpl-p95-"); os.makedirs(work, exist_ok=True)

    flat = [p for ports in by_problem.values() for p in ports]
    print(f"{len(by_problem)} problems, {len(flat)} candidate ports; building with {a.jobs} jobs", flush=True)
    def safe(p):
        try: return prepare(p, work, a.timeout, hand, hand_by_prob, audit2)
        except Exception as e: return {"problem": p["problem"], "port": p["path"], "skip": f"harness error: {e!r}"}
    with concurrent.futures.ThreadPoolExecutor(a.jobs) as ex:
        prepared = {r["port"]: r for r in ex.map(safe, flat)}
    built_sizes = {r["port"]: r["emitted_bytes"] for r in prepared.values() if "emitted_bytes" in r}

    rows = []
    for name in sorted(by_problem):
        chosen = None
        for p in by_problem[name]:
            r = prepared[p["path"]]
            if "skip" not in r:
                measure(r, a.runs, a.timeout)
            rows.append(r)
            if "time_ratio" in r:
                chosen = r
                print(f"{r['time_ratio']:7.2f}x {r['orig_s'] * 1e3:8.1f} -> {r['port_s'] * 1e3:8.1f} ms  rss {r['rss_ratio']:5.2f}x  {r['emitted_bytes']:6d} B  {r['port']}  [{r['input']}]", flush=True)
                break
        if not chosen:
            print(f"   skip  {name}: {rows[-1]['skip'][:120]}", flush=True)
    for r in rows:
        r.pop("dir", None); r.pop("input_path", None)

    s = summarise(rows, built_sizes)
    meta = {"commit": sh(["git", "-C", ROOT, "rev-parse", "--short", "HEAD"]).stdout.strip(),
            "cpl_version": sh([CPL, "--version"]).stdout.strip().split()[-1],
            "gxx": sh(["g++", "--version"]).stdout.splitlines()[0], "runs": a.runs, "problems": len(by_problem),
            "cpu": next((l.split(":", 1)[1].strip() for l in open("/proc/cpuinfo") if l.startswith("model name")), platform.machine()),
            "cores": os.cpu_count()}
    text = report(meta, s, rows)
    print("\n" + text.split("## Top 10")[0])
    if not a.no_write and not a.ports:
        with open(OUT_JSON, "w") as f: json.dump({"meta": meta, "summary": s, "ports": rows}, f, indent=1); f.write("\n")
        with open(OUT_MD, "w") as f: f.write(text)
        print(f"wrote {os.path.relpath(OUT_JSON, ROOT)} and {os.path.relpath(OUT_MD, ROOT)}")
    if not a.keep: shutil.rmtree(work, ignore_errors=True)
    return 0 if s["target_met"] or not a.check else 1


if __name__ == "__main__":
    sys.exit(main())
