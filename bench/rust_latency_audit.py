#!/usr/bin/env python3
"""Toolchain latency audit of the Rust `cpl` binary against the P0 budgets.

    check < 50 ms | lsp warm diagnostics and hover < 100 ms | run < 200 ms
    build < 1 s and < 64 KB | stress first case < 1 s

Every command is a fresh process, timed end to end (spawn to exit), repeated
`--reps` times; the median is reported per file and the worst file per command
is named. Sources come from the port set of rust_max_n_audit.py (real
solutions) plus the CLI's own test programs (the only files with small inputs
for `cpl run`).

    cargo build --release -p cpl-cli
    python3 bench/rust_latency_audit.py [--reps 9] [--json out.json]
"""
import argparse, glob, json, os, shutil, statistics, subprocess, sys, tempfile, time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import rust_max_n_audit as maxn

ROOT, CPL = maxn.ROOT, maxn.CPL
BUDGET_MS = {"check": 50, "run": 200, "build": 1000, "lsp diagnostics": 100, "lsp hover": 100, "stress first case": 1000}


def ms(cmd, stdin=None, reps=1):
    ts = []
    for _ in range(reps):
        t0 = time.perf_counter()
        p = subprocess.run(cmd, stdin=stdin and open(stdin), stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        ts.append((time.perf_counter() - t0) * 1e3)
    return statistics.median(ts), p.returncode


class Lsp:
    def __init__(self):
        self.p = subprocess.Popen([CPL, "lsp"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
        self.id = 0
        self.request("initialize", {"processId": None, "rootUri": None, "capabilities": {}})
        self.notify("initialized", {})

    def send(self, msg):
        body = json.dumps({"jsonrpc": "2.0", **msg}).encode()
        self.p.stdin.write(b"Content-Length: %d\r\n\r\n" % len(body) + body); self.p.stdin.flush()

    def recv(self):
        n = 0
        while True:
            line = self.p.stdout.readline()
            if not line: raise EOFError("language server exited")
            if line.lower().startswith(b"content-length:"): n = int(line.split(b":")[1])
            if line in (b"\r\n", b"\n"): break
        return json.loads(self.p.stdout.read(n))

    def notify(self, method, params):
        self.send({"method": method, "params": params})

    def request(self, method, params):
        self.id += 1; self.send({"id": self.id, "method": method, "params": params})
        while True:
            m = self.recv()
            if m.get("id") == self.id: return m

    def diagnostics(self):
        while True:
            m = self.recv()
            if m.get("method") == "textDocument/publishDiagnostics": return m["params"]["diagnostics"]

    def close(self):
        try:
            self.request("shutdown", None); self.notify("exit", None); self.p.wait(timeout=5)
        except Exception:
            self.p.kill()


def lsp_times(path, reps):
    """Warm server: time from an edit to its diagnostics, and a hover round trip."""
    text = open(path).read(); uri = "file://" + path
    lines = text.split("\n")
    s = Lsp(); doc = {"uri": uri}
    t0 = time.perf_counter()
    s.notify("textDocument/didOpen", {"textDocument": {**doc, "languageId": "cp", "version": 1, "text": text}})
    s.diagnostics(); cold = (time.perf_counter() - t0) * 1e3
    end = {"line": len(lines) - 1, "character": len(lines[-1])}
    diag, hover = [], []
    # hover a few identifiers spread through the file
    spots = [(i, len(l) - len(l.lstrip()) + 1) for i, l in enumerate(lines) if l.strip() and not l.strip().startswith("//")]
    spots = spots[:: max(1, len(spots) // reps)][:reps]
    for v in range(reps):
        # type a comment at the end of the file, then delete it: two real incremental edits
        t0 = time.perf_counter()
        s.notify("textDocument/didChange", {"textDocument": {**doc, "version": 2 * v + 2},
                                             "contentChanges": [{"range": {"start": end, "end": end}, "text": "\n// x"}]})
        s.diagnostics(); diag.append((time.perf_counter() - t0) * 1e3)
        t0 = time.perf_counter()
        s.notify("textDocument/didChange", {"textDocument": {**doc, "version": 2 * v + 3},
                                             "contentChanges": [{"range": {"start": end, "end": {"line": end["line"] + 1, "character": 4}}, "text": ""}]})
        s.diagnostics(); diag.append((time.perf_counter() - t0) * 1e3)
        line, ch = spots[v % len(spots)]
        t0 = time.perf_counter()
        s.request("textDocument/hover", {"textDocument": doc, "position": {"line": line, "character": ch}})
        hover.append((time.perf_counter() - t0) * 1e3)
    s.close()
    return cold, statistics.median(diag), max(diag), statistics.median(hover), max(hover)


def summarize(name, rows, out):
    """rows: [(file, ms)]"""
    if not rows: return
    v = sorted(x for _, x in rows); worst = max(rows, key=lambda r: r[1]); b = BUDGET_MS[name]
    over = sum(x >= b for x in v)
    out[name] = {"files": len(v), "median_ms": round(statistics.median(v), 2), "p90_ms": round(v[int(0.9 * (len(v) - 1))], 2),
                 "max_ms": round(worst[1], 2), "worst": worst[0], "budget_ms": b, "over_budget": over}
    print(f"{name:18s} budget {b:5d} ms | {len(v):3d} files | median {statistics.median(v):8.2f}  p90 {v[int(0.9 * (len(v) - 1))]:8.2f}  "
          f"max {worst[1]:8.2f} ({worst[0]}) | {'PASS' if not over else f'{over} OVER'}")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--reps", type=int, default=9)
    ap.add_argument("--json")
    a = ap.parse_args()
    if not os.path.exists(CPL): sys.exit(f"{CPL} not found: cargo build --release -p cpl-cli")
    tmp = tempfile.mkdtemp(prefix="cpl-lat-")
    ports = []
    for spec in maxn.SPECS:                       # copies: `cpl build` writes next to its input by default
        dst = os.path.join(tmp, spec[0].replace("/", "__") + ".cp"); shutil.copy(maxn.P(spec[0] + ".cp"), dst); ports.append((spec, dst))
    out = {"sizes": {}}

    rows = {k: [] for k in BUDGET_MS}; sizes = []
    for spec, src in ports:
        name = spec[0]
        t, rc = ms([CPL, "check", src], reps=a.reps)
        if rc: print(f"  note: cpl check exits {rc} on {name}")
        rows["check"].append((name, t))
        cpp = src[:-3] + ".cpp"
        t, rc = ms([CPL, "build", src, "-o", cpp], reps=a.reps); rows["build"].append((name, t))
        sizes.append((name, os.path.getsize(cpp)))
        _, d, dmax, h, hmax = lsp_times(src, a.reps)
        rows["lsp diagnostics"].append((name, dmax)); rows["lsp hover"].append((name, hmax))

    # `cpl run`: the CLI's golden programs with their inputs, and each port on its medium input
    for inp in sorted(glob.glob(os.path.join(ROOT, "crates/cpl-cli/tests/programs/*.in"))):
        src = inp[:-3] + ".cp"
        if os.path.exists(src):
            t, rc = ms([CPL, "run", src], stdin=inp, reps=a.reps); rows["run"].append(("tests/" + os.path.basename(src), t))
    ctx = {"run_orig": None}
    run_medium = []
    for spec, src in ports:
        if spec[0].endswith("noiwc22p3"): continue   # its input needs the compiled original
        inp = src[:-3] + ".in"; spec[4](inp, False, ctx)
        t0 = time.perf_counter()
        try:
            p = subprocess.run([CPL, "run", src], stdin=open(inp), stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=30)
            run_medium.append((spec[0], (time.perf_counter() - t0) * 1e3, os.path.getsize(inp), p.returncode))
        except subprocess.TimeoutExpired:
            run_medium.append((spec[0], 30000.0, os.path.getsize(inp), "timeout"))

    # `cpl stress --cases 1` against the original C++ as the brute: cold (empty binary cache, so both
    # g++ builds are paid) and again warm. Only runs that really compared a case count.
    stress = []
    for spec, src in ports:
        if spec[0].split("/")[-1] in ("noiwc22p3", "ioi19p5", "noi04p3", "noi08p4"): continue   # graders / inexact answers
        env = {**os.environ, "CPL_CACHE": os.path.join(tmp, "cache-" + os.path.basename(src))}
        cmd = [CPL, "stress", src, "--brute", spec[1], "--cases", "1"]
        t = []
        for _ in range(2):
            t0 = time.perf_counter(); p = subprocess.run(cmd, capture_output=True, text=True, env=env); t.append((time.perf_counter() - t0) * 1e3)
        msg = (p.stdout + p.stderr)
        kind = ("agree" if "agree" in msg and p.returncode == 0 else "no generator for a read() type" if "can't generate" in msg
                else "brute crashed on the generated input" if "failed on this input" in msg else "other: " + msg.strip().splitlines()[0][:80])
        stress.append((spec[0], t[0], t[1], kind))
        shutil.rmtree(src[:-3] + ".tests", ignore_errors=True)
    ran = [s for s in stress if "no generator" not in s[3]]
    rows["stress first case"] = [(n, t1) for n, t1, _, _ in ran]

    for k in BUDGET_MS: summarize(k, rows[k], out)
    big = max(sizes, key=lambda s: s[1])
    out["sizes"] = {"max_bytes": big[1], "worst": big[0], "median_bytes": statistics.median(s for _, s in sizes), "over_64k": sum(s >= 65536 for _, s in sizes)}
    print(f"emitted size       budget 65536 B  | {len(sizes):3d} files | median {out['sizes']['median_bytes']:.0f}  max {big[1]} ({big[0]})")
    slow = sorted(run_medium, key=lambda r: -r[1])
    out["run_medium"] = [{"port": n, "ms": round(t, 1), "input_bytes": b, "exit": rc} for n, t, b, rc in slow]
    print(f"\ncpl run on medium inputs (interpreter throughput, no budget): median {statistics.median(t for _, t, _, _ in slow):.0f} ms; slowest:")
    for n, t, b, rc in slow[:6]: print(f"  {t:9.0f} ms  {n}  ({b} B input, exit {rc})")
    out["stress"] = [{"port": n, "cold_ms": round(a1, 1), "warm_ms": round(a2, 1), "result": k} for n, a1, a2, k in stress]
    kinds = {}
    for s in stress: kinds[s[3]] = kinds.get(s[3], 0) + 1
    print(f"cpl stress --cases 1 on {len(stress)} ports: {kinds}; warm (binaries cached) median "
          f"{statistics.median(s[2] for s in ran):.0f} ms, max {max(s[2] for s in ran):.0f} ms")
    if a.json:
        out["rows"] = {k: [{"file": n, "ms": round(t, 2)} for n, t in v] for k, v in rows.items()}
        json.dump(out, open(a.json, "w"), indent=1)
    shutil.rmtree(tmp, ignore_errors=True)


if __name__ == "__main__":
    main()
