#!/usr/bin/env python3
"""Benchmark every corpus port against the accepted C++ it re-solves.

For each proto/ports/<name>.cp this builds the port to C++, compiles it and the original with
`g++ -std=c++17 -O2`, and reports

  correctness   outputs agree on every measured input (interactive ports: both pass the judge)
  lines         port lines vs the original, raw and library-stripped (proto/loc.py, <name>.strip)
  emitted C++   bytes of the generated .cpp, and the stripped binary against the original's
  runtime       wall time on the largest generated inputs the original handles within --limit
  peak RSS      maximum resident set size of the same runs

Inputs come from the port's own typed generator (`<name>_gen seed size`), so every input meets
the port's `where` preconditions. Results go to results.json and results.md next to this file.

usage: bench.py [--ports a,b,...] [--seeds 3] [--limit 2.0] [--out DIR]
"""
import argparse, importlib.util, json, os, random, resource, signal, subprocess, sys, tempfile, threading, time

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
PROTO = os.path.join(ROOT, "proto")
PORTS = os.path.join(PROTO, "ports")
PORT_DIRS = ["", "hillclimb_dp_graph"]               # the folders of ports proto/run_all.sh checks
SIZES = [1000000, 300000, 100000, 30000, 10000, 3000, 1000, 300, 100, 30, 10, 6]   # generator size caps
REJECTED = 4                                         # generator exit code: a `where` failed
GNU_TIME = "/usr/bin/time"                           # for peak RSS
GEN_SECONDS, GEN_MEMORY, GEN_BYTES = 5, 4 << 30, 64 << 20   # what producing one input may cost


def load(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


loc = load(os.path.join(PROTO, "loc.py"), "loc")
interact = load(os.path.join(PROTO, "interact.py"), "interact")


def unlimited_stack():
    """some originals keep megabytes of tables on the stack"""
    try:
        resource.setrlimit(resource.RLIMIT_STACK, (resource.RLIM_INFINITY, resource.RLIM_INFINITY))
    except (ValueError, OSError):
        pass


def timed(exe, text, limit):
    """(seconds, peak RSS in KB, stdout) of one run, or None if it fails or exceeds `limit`.

    The run goes through GNU time: a child's own rusage also counts the memory of the Python
    process it was forked from, which would hide the footprint of small programs."""
    with tempfile.NamedTemporaryFile("r") as rss, tempfile.TemporaryFile() as fin, tempfile.TemporaryFile() as fout:
        fin.write(text.encode()); fin.seek(0)
        start = time.perf_counter()
        p = subprocess.Popen([GNU_TIME, "-f", "%M", "-o", rss.name, exe], stdin=fin, stdout=fout,
                             stderr=subprocess.DEVNULL, start_new_session=True)
        # a blocking wait with a watchdog: `wait(timeout=...)` polls, which rounds short runs up
        watchdog = threading.Timer(limit, lambda: os.killpg(p.pid, signal.SIGKILL))
        watchdog.start()
        p.wait()
        took = time.perf_counter() - start
        watchdog.cancel()
        if p.returncode != 0:
            return None
        fout.seek(0)
        return took, int(rss.read().split()[-1]), fout.read().decode(errors="replace")


def compile_cpp(src, out, *flags):
    r = subprocess.run(["g++", "-std=c++17", "-O2", *flags, "-o", out, src], capture_output=True, text=True)
    if r.returncode:
        raise RuntimeError(f"g++ failed on {os.path.basename(src)}:\n{r.stderr[:2000]}")


def stripped_size(exe):
    out = exe + ".stripped"
    subprocess.run(["strip", "-o", out, exe], check=True)
    size = os.path.getsize(out)
    os.remove(out)
    return size


class TooBig(Exception):
    """the generator cannot produce an input at this size cap in reasonable time or memory"""


def generator_limits():
    resource.setrlimit(resource.RLIMIT_AS, (GEN_MEMORY, GEN_MEMORY))


def generate(gen, seed, size):
    """one generated input, or None when it broke a `where` precondition"""
    try:
        g = subprocess.run([gen, str(seed), str(size)], capture_output=True, text=True,
                           timeout=GEN_SECONDS, preexec_fn=generator_limits)
    except subprocess.TimeoutExpired:
        raise TooBig()
    if g.returncode == REJECTED:
        return None
    if g.returncode != 0 or len(g.stdout) > GEN_BYTES:
        raise TooBig()
    return g.stdout


def inputs_at(gen, size, want):
    """the first `want` accepted inputs at this size cap (seeds 1, 2, ...); none if the cap is too big"""
    found = []
    for seed in range(1, 40 * want + 1):
        try:
            text = generate(gen, seed, size)
        except TooBig:
            return []
        if text is not None:
            found.append((seed, text))
            if len(found) == want:
                break
    return found


def bench_port(port, work, seeds, limit):
    """`port` is a name in proto/ports, or `folder/name` for a port in one of its subfolders"""
    folder, name = os.path.split(port)
    folder = os.path.join(PORTS, folder)
    cp, orig = os.path.join(folder, name + ".cp"), os.path.join(folder, name + "_orig.cpp")
    judge = os.path.join(folder, name + "_judge.py")
    r = subprocess.run([sys.executable, os.path.join(PROTO, "cp.py"), "build", cp], capture_output=True, text=True)
    if r.returncode:
        raise RuntimeError(f"cp.py build failed:\n{r.stdout}{r.stderr}")
    emitted = os.path.join(folder, name + ".cpp")
    ours, gen, ref = (os.path.join(work, f"{name}_{k}") for k in ("ours", "gen", "ref"))
    compile_cpp(emitted, ours)
    compile_cpp(emitted, gen, "-DCP_GEN_MODE")
    compile_cpp(orig, ref)

    loc.PORTS = folder
    port_lines, raw_lines, stripped_lines, test_lines = loc.measure(name)
    row = {
        "port": port, "interactive": os.path.exists(judge),
        "port_lines": port_lines, "port_test_lines": test_lines,
        "orig_lines": raw_lines, "orig_stripped_lines": stripped_lines,
        "raw_reduction": 1 - port_lines / raw_lines,
        "stripped_reduction": 1 - port_lines / stripped_lines,
        "emitted_cpp_bytes": os.path.getsize(emitted), "orig_cpp_bytes": os.path.getsize(orig),
        "ours_binary_bytes": stripped_size(ours), "orig_binary_bytes": stripped_size(ref),
    }

    if row["interactive"]:
        mod = load(judge, "judge_" + name)
        talks = 0
        for seed in range(1, 20 * seeds + 1):
            text = generate(gen, seed, 100)
            if text is None:
                continue
            for exe in (ours, ref):
                if not interact.talk(exe, mod.Judge(text, random.Random(seed))):
                    raise RuntimeError(f"judge rejected {os.path.basename(exe)} on seed {seed}")
            talks += 1
        row.update(agree=talks > 0, inputs=talks, size=100)
        return row

    # the largest size cap at which the original still answers within the limit
    for size in SIZES:
        cases = inputs_at(gen, size, seeds)
        if cases and timed(ref, cases[0][1], limit) is not None:
            break
    else:
        raise RuntimeError("the original fails even on the smallest generated inputs")
    agree, runs = True, []
    for seed, text in cases:
        a, b = timed(ours, text, 10 * limit), timed(ref, text, 10 * limit)
        if a is None or b is None or a[2].split() != b[2].split():
            agree = False
            continue
        runs.append((a[0], b[0], a[1], b[1]))
    if runs:
        row.update(ours_seconds=max(r[0] for r in runs), orig_seconds=max(r[1] for r in runs),
                   ours_rss_kb=max(r[2] for r in runs), orig_rss_kb=max(r[3] for r in runs))
    row.update(agree=agree and bool(runs), inputs=len(cases), size=size)
    return row


def markdown(rows):
    def pct(x): return f"{100 * x:.0f}%"
    out = ["| port | lines | orig | raw | stripped orig | stripped | emitted C++ | binary vs orig | ours s | orig s | ours MB | orig MB | agree |",
           "|---|---|---|---|---|---|---|---|---|---|---|---|---|"]
    for r in rows:
        if "error" in r:
            out.append(f"| `{r['port']}` | failed: {r['error'].splitlines()[0][:80]} |" + " |" * 11)
            continue
        if "ours_seconds" in r:
            perf = (f"{r['ours_seconds']:.2f} | {r['orig_seconds']:.2f} | "
                    f"{r['ours_rss_kb'] / 1024:.0f} | {r['orig_rss_kb'] / 1024:.0f}")
        else:
            perf = "judge | judge | | " if r["interactive"] else " | | | "
        out.append(f"| `{r['port']}` | {r['port_lines']} | {r['orig_lines']} | {pct(r['raw_reduction'])} | "
                   f"{r['orig_stripped_lines']} | {pct(r['stripped_reduction'])} | {r['emitted_cpp_bytes'] / 1024:.1f} KB | "
                   f"{r['ours_binary_bytes'] / r['orig_binary_bytes']:.2f}x | {perf} | {'yes' if r['agree'] else 'NO'} |")
    ok = [r for r in rows if "error" not in r]
    if ok:
        port, raw, stripped = (sum(r[k] for r in ok) for k in ("port_lines", "orig_lines", "orig_stripped_lines"))
        timed_rows = [r for r in ok if "ours_seconds" in r]
        out += ["", f"{len(ok)} ports: {port} port lines against {raw} original lines ({pct(1 - port / raw)} fewer) "
                    f"and {stripped} library-stripped lines ({pct(1 - port / stripped)} fewer)."]
        if timed_rows:
            slow = sorted(r["ours_seconds"] / max(r["orig_seconds"], 1e-3) for r in timed_rows)
            mem = sorted(r["ours_rss_kb"] / r["orig_rss_kb"] for r in timed_rows)
            out.append(f"Runtime against the original (`g++ -O2`, worst of the measured inputs): median "
                       f"{slow[len(slow) // 2]:.2f}x, worst {slow[-1]:.2f}x. Peak RSS: median {mem[len(mem) // 2]:.2f}x, "
                       f"worst {mem[-1]:.2f}x. Outputs agree on {sum(r['agree'] for r in ok)} of {len(ok)} ports.")
    return "\n".join(out) + "\n"


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--ports", help="comma-separated ports, `name` or `folder/name` (default: all)")
    ap.add_argument("--seeds", type=int, default=3, help="inputs measured per port")
    ap.add_argument("--limit", type=float, default=2.0, help="seconds the original may take on an input")
    ap.add_argument("--out", default=os.path.dirname(os.path.abspath(__file__)), help="where results.json and results.md go")
    args = ap.parse_args()
    if not os.path.exists(GNU_TIME):
        sys.exit(f"bench.py needs GNU time at {GNU_TIME} to measure peak RSS")
    unlimited_stack()   # inherited by everything measured
    names = args.ports.split(",") if args.ports else [
        os.path.join(d, f[:-3]) for d in PORT_DIRS for f in sorted(os.listdir(os.path.join(PORTS, d))) if f.endswith(".cp")]
    rows = []
    with tempfile.TemporaryDirectory() as work:
        for name in names:
            try:
                row = bench_port(name, work, args.seeds, args.limit)
            except Exception as e:   # one broken port must not hide the rest
                row = {"port": name, "error": str(e)}
            rows.append(row)
            print(json.dumps(row), flush=True)
    os.makedirs(args.out, exist_ok=True)
    with open(os.path.join(args.out, "results.json"), "w") as f:
        json.dump(rows, f, indent=1)
        f.write("\n")
    with open(os.path.join(args.out, "results.md"), "w") as f:
        f.write(markdown(rows))
    sys.exit(1 if any("error" in r or not r["agree"] for r in rows) else 0)


if __name__ == "__main__":
    main()
