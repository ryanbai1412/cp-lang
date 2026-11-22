#!/usr/bin/env python3
"""Max-N judge audit of the Rust `cpl build` back end (audit #2).

For each port: `cpl build` -> one C++17 file, `g++ -std=c++17 -O2` on it and on
the accepted original, run both on a worst-case input at the judge limits, and
report output equality, wall-time ratio, peak-RSS ratio and emitted size
(limit 64 KB). The emitted C++ is also built with ASan/UBSan and run on a
medium input. Tracked files are never written: everything goes to a temp dir.

    cargo build --release -p cpl-cli
    python3 bench/rust_max_n_audit.py                 # all ports, 3 runs each
    python3 bench/rust_max_n_audit.py --ports dpl,noi22p5 --runs 5
    python3 bench/rust_max_n_audit.py --json out.json --list

Write-up: research/max-n-judge-audit-2-rust.md
"""
import argparse, json, os, random, resource, signal, statistics, subprocess, sys, tempfile, threading, time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PORTS = os.path.join(ROOT, "proto", "ports")
CORPUS = os.path.join(ROOT, "corpus", "submissions")
CPL = os.environ.get("CPL") or os.path.join(ROOT, "target", "release", "cpl")
SIZE_LIMIT = 64 * 1024
TIMEOUT = 60

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(PORTS, "blind_critic_5"))
os.environ.setdefault("CPL", CPL)
import max_n_audit as audit1          # worst-case generators shared with audit #1
import check as bc5                   # grader drivers for the Blind Critic #5 ports


def W(path, lines):
    with open(path, "w") as f:
        f.write("\n".join(map(str, lines)) + "\n")


def J(xs):
    return " ".join(map(str, xs))


def letters(rng, n, alphabet="ab"):
    return "".join(rng.choice(alphabet) for _ in range(n))


# ---- worst-case generators: gen(path, big, ctx). `big` picks the judge limit,
# otherwise a medium size for the sanitizer run. -------------------------------

def g_dpk(p, big, _):
    n, k = (100, 100000) if big else (50, 5000)
    W(p, [f"{n} {k}", J(range(1, n + 1))])

def g_dpl(p, big, _):
    rng = random.Random(1); n = 3000 if big else 400
    W(p, [n, J(rng.randint(1, 10**9) for _ in range(n))])

def g_dpx(p, big, _):
    rng = random.Random(2); n = 1000 if big else 200
    W(p, [n] + [f"{rng.randint(1, 10**4)} {rng.randint(1, 10**4)} {rng.randint(1, 10**9)}" for _ in range(n)])

def g_dpy(p, big, _):
    rng = random.Random(3); h = w = 10**5 if big else 500; n = 3000 if big else 300
    cells = set()
    while len(cells) < n:
        c = (rng.randint(1, h), rng.randint(1, w))
        if c not in ((1, 1), (h, w)): cells.add(c)
    W(p, [f"{h} {w} {n}"] + [f"{r} {c}" for r, c in sorted(cells)])

def g_dpf(p, big, _):
    rng = random.Random(4); n = 3000 if big else 300
    W(p, [letters(rng, n, "abc"), letters(rng, n, "abc")])

def g_btoi19p4(p, big, _):
    rng = random.Random(5); n = m = 300 if big else 40; k = 20 if big else 5
    W(p, [f"{n} {m} {k}", J(rng.randint(k, 300) for _ in range(n)), J(rng.randint(250, 300) for _ in range(m))])

def g_noip19p4(p, big, _):
    rng = random.Random(6); n, m = (100, 2000) if big else (30, 60)
    W(p, [f"{n} {m}"] + [J(rng.randint(0, 998244352) for _ in range(m)) for _ in range(n)])

def g_coci21c2p4(p, big, _):
    rng = random.Random(7); n, l = (50, 10000) if big else (15, 600)
    W(p, [f"{n} {l}", J(rng.randint(1, max(1, l // (2 * n))) for _ in range(n))])

def g_noip14p5(p, big, _):
    rng = random.Random(8); n, m = (10000, 200000) if big else (500, 5000)
    W(p, [f"{n} {m}"] + [f"{rng.randint(1, n)} {rng.randint(1, n)}" for _ in range(m)] + [f"1 {n}"])

def g_noi07p1(p, big, _):
    rng = random.Random(9); n = 100 if big else 30
    es = [(i, j) for i in range(1, n + 1) for j in range(i + 1, n + 1)]
    if not big: es = [e for e in es if e[1] == e[0] + 1 or rng.random() < 0.3]
    es = es[:4500]
    W(p, [f"{n} {len(es)}"] + [f"{a} {b} {rng.randint(1, 3)}" for a, b in es])

def g_coci20c6p2(p, big, _):
    rng = random.Random(10); n = m = q = 1000 if big else 100
    names = [f"w{i:04d}" for i in range(n)]
    W(p, [f"{n} {m}"] + [f"{rng.choice(names)} {rng.choice(names)} {rng.randint(1, 10**9)}" for _ in range(m)]
      + [q] + [f"{rng.choice(names)} {rng.choice(names)}" for _ in range(q)])

def g_noip10p4(p, big, _):
    n = m = 500 if big else 60          # one long snake: every cell drains to the next
    rows = []
    for r in range(n):
        base = (n - r) * m
        row = [base - c for c in range(m)]
        rows.append(J(row if r % 2 == 0 else row[::-1]))
    W(p, [f"{n} {m}"] + rows)

def g_noip19p2(p, big, _):
    rng = random.Random(12); n = 500000 if big else 5000
    s = "".join("(" if rng.random() < 0.55 else ")" for _ in range(n))
    W(p, [n, s, J(max(1, i - rng.randint(1, 3)) for i in range(2, n + 1))])

def g_btoi17p6(p, big, _):
    rng = random.Random(13); n = 200000 if big else 5000
    W(p, [f"{n} 3"] + [max(0, i - rng.randint(1, 4)) for i in range(1, n)])

def g_shoi16p1(p, big, _):
    rng = random.Random(14); n, k = (500000, 20) if big else (3000, 6)
    marked = sorted(rng.sample(range(1, n + 1), n // 2))
    W(p, [f"{n} {k}", J(rng.randint(1, 1000) for _ in range(n)), len(marked), J(marked)]
      + [f"{rng.randint(max(1, i - 5), i - 1)} {i}" for i in range(2, n + 1)])

def g_noi09p1(p, big, _):
    n = 10000 if big else 500           # d[i] = 1 everywhere: one big cycle of choices
    W(p, [n, J([1] * n)])

def g_dmpg15g6(p, big, _):
    rng = random.Random(16); n = q = 100000 if big else 2000; m = n
    W(p, [f"{n} {m}", J(rng.randint(1, m) for _ in range(n)), q]
      + [f"{l} {r}" for l, r in (sorted((rng.randint(1, n), rng.randint(1, n))) for _ in range(q))])

def g_ioi05p1(p, big, _):
    rng = random.Random(17); l = w = 250 if big else 40; n = 5000 if big else 200; k = 20 if big else 4
    W(p, [f"{l} {w}", f"{n} {k}"] + [f"{rng.randint(1, l)} {rng.randint(1, w)}" for _ in range(n)])

def g_ioi09p5(p, big, _):
    # only parked cars leave (the statement's guarantee), and the garage stays full with a queue
    rng = random.Random(18); n, m = (100, 2000) if big else (10, 200)
    ev, parked, queue, nxt = [], [], [], 1
    while len(ev) < 2 * m:
        if nxt <= m and (len(parked) < n or not parked or rng.random() < 0.6):
            ev.append(nxt); (parked if len(parked) < n else queue).append(nxt); nxt += 1
        else:
            ev.append(-parked.pop(rng.randrange(len(parked))))
            if queue: parked.append(queue.pop(0))
    W(p, [f"{n} {m}"] + [rng.randint(1, 100) for _ in range(n)] + [rng.randint(1, 10000) for _ in range(m)] + ev)

def g_saspeed(p, big, _):
    rng = random.Random(19); n = 10**6 if big else 20000
    W(p, [letters(rng, n, "ab")])

def g_noip20p2(p, big, _):
    rng = random.Random(20); t, n = (5, 1 << 20) if big else (3, 5000)
    W(p, [t] + [("ab" * (n // 2)) if i % 2 == 0 else letters(rng, n, "ab") for i in range(t)])

def g_btoi19p5(p, big, _):
    rng = random.Random(21); n = 3000 if big else 300
    W(p, [letters(rng, n, "ab"), letters(rng, n, "ab")])

def g_sdoi17r1p1(p, big, _):
    rng = random.Random(22); t, hi = (1000, 10**6) if big else (50, 20000)
    W(p, [t] + [f"{rng.randint(hi * 9 // 10, hi)} {rng.randint(hi * 9 // 10, hi)}" for _ in range(t)])

def g_noi14p1(p, big, _):
    rng = random.Random(23); n = 100000 if big else 2000
    W(p, [f"{n} 1000000000"] + [f"{rng.choice(['AND', 'OR', 'XOR'])} {rng.randint(0, 10**9)}" for _ in range(n)])

def g_noip14p6(p, big, _):
    rng = random.Random(24); n, m, d = (100, 10**6, 10000) if big else (20, 20000, 300)
    def num():
        return ("-" if rng.random() < 0.5 else "") + str(rng.randint(1, 9)) + "".join(str(rng.randint(0, 9)) for _ in range(d - 1))
    W(p, [f"{n} {m}"] + [num() for _ in range(n + 1)])

def g_ccc18s4(p, big, _):
    W(p, [10**9 if big else 10**6])

def g_ccc23s5(p, big, _):
    W(p, [10**9 if big else 10**6])

def g_noip05p2(p, big, _):
    rng = random.Random(27); m = 100
    W(p, [10**9 if big else 10**5, "9 10", m, J(sorted(rng.sample(range(1, 10**5), m)))])

def shared(name):
    fn = getattr(audit1, "gen_" + name)
    return lambda p, big, _: fn(p)

# Blind Critic #5 (limits from the statements; bsspc21s4's are inferred)
def g_bsspc21s4(p, big, _):
    rng = random.Random(31); n = m = 10**9 if big else 1000; k = 100000 if big else 500
    rows = []
    for _ in range(k):
        x1, x2 = sorted((rng.randint(1, n), rng.randint(1, n))); y1, y2 = sorted((rng.randint(1, m), rng.randint(1, m)))
        rows.append(f"{x1} {y1} {x2} {y2}")
    W(p, [f"{n} {m} {k}"] + rows)

def g_noi22p5(p, big, _):
    # values stay <= n: the original mis-sizes a segment tree when there are more distinct values
    rng = random.Random(32); n = m = 200000 if big else 2000
    rows = []
    for _ in range(m):
        l = rng.randint(1, n); r = min(n, l + rng.randint(0, 50))
        rows.append(f"{l} {r} {max(1, min(n, l // 2 + rng.randint(0, 3)))}")
    W(p, [1, f"{n} {m}"] + rows)

def g_noi04p3(p, big, _):
    rng = random.Random(33); n, m, q = (10, 100, 100) if big else (5, 20, 10)
    W(p, [f"{n} {m}", letters(rng, n, "EW"), letters(rng, m, "NS"), J(rng.randint(1, 10000) for _ in range(n)),
          J(rng.randint(1, 10000) for _ in range(m)), q]
      + [f"{rng.randint(1, n)} {rng.randint(1, m)} {rng.randint(1, n)} {rng.randint(1, m)}" for _ in range(q)])

def g_noi08p4(p, big, _):
    rng = random.Random(34); n, m = (60, 30) if big else (20, 8)
    clen = n // 3
    par = [0] * n
    for i in range(clen): par[i] = (i + 1) % clen
    for i in range(clen, n): par[i] = rng.randint(max(0, i - 3), i - 1)
    W(p, [f"{n} {m} 0.5", J(x + 1 for x in par), J(round(rng.uniform(1, 100000), 1) for _ in range(n))])

def g_noiwc22p3(p, big, ctx):
    # Real dictionary size (8869 words). Each game's later queries depend on the
    # solver's own guesses, so the transcript is grown one round at a time by
    # replaying it through the original.
    rng = random.Random(35); nw, games = (8869, 60) if big else (400, 8)
    words = {"leant", "paper"}
    while len(words) < nw: words.add(letters(rng, 5, "aeilnrstop"))
    words = sorted(words)
    secrets = [rng.choice(words) for _ in range(games)]
    q = [[f"5 {s[0]} 00000 00000"] for s in secrets]
    head = [len(words)] + words
    for rnd in range(1, 5):
        flat = [x for g in q for x in g]
        W(p, head + [len(flat)] + flat)
        out = ctx["run_orig"](p).split()
        i = 0
        for g, s in zip(q, secrets):
            last = out[i + len(g) - 1]; i += len(g)
            if last != s and len(g) == rnd:
                gold, silver = bc5.eval_wordle(last, s)
                g.append(f"{5 - rnd} {s[0]} {gold} {silver}")
    flat = [x for g in q for x in g]
    W(p, head + [len(flat)] + flat)

def g_ioi19p5(p, big, _):
    W(p, ["200 200 199" if big else "12 9 7"])


# (port under corpus/problems, original, topic, limits, generator, compare mode)
def O(n): return os.path.join(CORPUS, f"{n}.cpp")
def P(rel): return os.path.join(PORTS, rel)
SPECS = [
    ("blind_critic_5/bsspc21s4", O(4152932), "sweep+segtree", "N=M=1e9, K=1e5 (inferred)", g_bsspc21s4, "exact"),
    ("blind_critic_5/noi22p5", O(4832632), "sweep+lazy segtree", "T=1, n=m=2e5", g_noi22p5, "exact"),
    ("blind_critic_5/noi04p3", O(5032868), "bitmask DP", "n=10, m=100, q=100", g_noi04p3, "head2"),
    ("blind_critic_5/noi08p4", O(5144322), "float tree DP", "n=60, m=30", g_noi08p4, "float"),
    ("blind_critic_5/noiwc22p3", O(4261177), "string filter", "8869 words, 60 games", g_noiwc22p3, "exact"),
    ("blind_critic_5/ioi19p5", O(5664246), "construction", "200x200, K=199", g_ioi19p5, "exact"),
    ("dpk", P("dpk_orig.cpp"), "DP", "N=100, K=1e5", g_dpk, "exact"),
    ("dpl", P("dpl_orig.cpp"), "DP", "N=3000", g_dpl, "exact"),
    ("dpx", P("dpx_orig.cpp"), "DP", "N=1000", g_dpx, "exact"),
    ("scout_batch_8/dpy", O(4133511), "DP+comb", "H=W=1e5, N=3000", g_dpy, "exact"),
    ("scout_batch_38/today/dpf", O(4131645), "DP+string", "|s|=|t|=3000", g_dpf, "len"),
    ("ioi04p4", P("ioi04p4_orig.cpp"), "DP", "W=H=600, N=200", shared("ioi04p4"), "exact"),
    ("scout_batch_20/btoi19p4", O(4456166), "knapsack", "N=M=300", g_btoi19p4, "exact"),
    ("scout_batch_29/noip19p4", O(5147491), "DP mod", "n=100, m=2000", g_noip19p4, "exact"),
    ("scout_batch_11/coci21c2p4", O(4226760), "DP mod", "n=50, l=1e4", g_coci21c2p4, "exact"),
    ("noip14p5", P("noip14p5_orig.cpp"), "graph BFS", "n=1e4, m=2e5", g_noip14p5, "exact"),
    ("scout_batch_2/apio09p3", P("hillclimb_dp_graph/apio09p3_orig.cpp"), "graph SCC", "N=M=5e5", shared("apio09p3"), "exact"),
    ("scout_batch_2/ioi11p4", P("hillclimb_dp_graph/ioi11p4_orig.cpp"), "graph Dijkstra", "N=1e5, M=1e6", shared("ioi11p4"), "exact"),
    ("scout_batch_2/dpv", P("hillclimb_dp_graph/dpv_orig.cpp"), "tree reroot", "N=1e5", shared("dpv"), "exact"),
    ("scout_batch_27/noi07p1", O(5032737), "graph Floyd", "n=100, m=4500", g_noi07p1, "exact"),
    ("scout_batch_27/coci20c6p2", O(4188022), "graph+map", "n=m=q=1000", g_coci20c6p2, "exact"),
    ("scout_batch_37/today/noip10p4", O(5205786), "grid DFS", "500x500 snake", g_noip10p4, "exact"),
    ("dmopc21c9p4", P("dmopc21c9p4_orig.cpp"), "grid", "1000x1000", shared("dmopc21c9p4"), "exact"),
    ("scout_batch_26/noip19p2", O(5127828), "tree", "n=5e5", g_noip19p2, "exact"),
    ("btoi17p6", P("btoi17p6_orig.cpp"), "tree greedy", "n=2e5", g_btoi17p6, "exact"),
    ("scout_batch_45/today/shoi16p1", O(5445691), "tree DP", "n=5e5, k=20", g_shoi16p1, "exact"),
    ("scout_batch_15/noi09p1", O(5102817), "graph peeling", "n=1e4", g_noi09p1, "exact"),
    ("scout_batch_24/today/dmpg15g6", O(4253124), "Mo + rollback", "n=q=1e5", g_dmpg15g6, "exact"),
    ("scout_batch_12/ioi05p1", O(5034025), "2D prefix", "250x250, 5000 roses", g_ioi05p1, "exact"),
    ("cco20p1", P("cco20p1_orig.cpp"), "range/steps", "N=1e5", shared("cco20p1"), "exact"),
    ("scout_batch_33/ioi09p5", O(5106592), "heap sim", "n=100, m=2000", g_ioi09p5, "exact"),
    ("scout_batch_10/saspeed", O(4852075), "suffix array", "|s|=1e6", g_saspeed, "exact"),
    ("scout_batch_10/noip20p2", O(5445673), "Z-function", "T=5, |s|=2^20", g_noip20p2, "exact"),
    ("scout_batch_10/btoi19p5", O(4456167), "KMP", "|s|=|t|=3000", g_btoi19p5, "head1"),
    ("scout_batch_11/sdoi17r1p1", O(5463988), "number theory", "T=1000, n,m=1e6", g_sdoi17r1p1, "exact"),
    ("scout_batch_22/noi14p1", O(3595714), "bitwise", "n=1e5", g_noi14p1, "exact"),
    ("scout_batch_8/noip14p6", O(5445671), "hash mod", "n=100, m=1e6, 1e4 digits", g_noip14p6, "exact"),
    ("ccc18s4", P("ccc18s4_orig.cpp"), "math memo", "N=1e9", g_ccc18s4, "exact"),
    ("ccc23s5", P("ccc23s5_orig.cpp"), "math", "N=1e9", g_ccc23s5, "exact"),
    ("scout_batch_28/noip05p2", O(5156196), "DP compress", "L=1e9, M=100", g_noip05p2, "exact"),
]


def sh(cmd, **kw):
    return subprocess.run(cmd, capture_output=True, text=True, **kw)


def big_stack():
    resource.setrlimit(resource.RLIMIT_STACK, (resource.RLIM_INFINITY, resource.RLIM_INFINITY))


def gb_stack():
    resource.setrlimit(resource.RLIMIT_STACK, (1 << 30, 1 << 30))   # ASan refuses an unlimited stack


def timed(binary, inp, outp, pre=big_stack, env=None):
    """Run once; return (wall seconds, peak RSS KB, status string).

    Peak RSS comes from /usr/bin/time: a child forked straight from this
    script would report at least Python's own resident size.
    """
    with open(inp) as fi, open(outp, "w") as fo, open(outp + ".err", "w") as fe:
        t0 = time.perf_counter()
        pr = subprocess.Popen(["/usr/bin/time", "-f", "%M", "-o", outp + ".rss", binary], stdin=fi, stdout=fo, stderr=fe,
                              preexec_fn=pre, env=env, start_new_session=True)
        killer = threading.Timer(TIMEOUT, lambda: os.killpg(pr.pid, signal.SIGKILL))
        killer.start()
        pr.wait()                                  # a blocking wait: a timeout here would poll and quantize the wall time
        wall = time.perf_counter() - t0
        timed_out = not killer.is_alive(); killer.cancel()
    if timed_out:
        return TIMEOUT, 0, "timeout"
    try:
        rss = int(open(outp + ".rss").read().split()[-1])
    except (OSError, ValueError, IndexError):
        rss = 0
    return wall, rss, "ok" if pr.returncode == 0 else f"exit {pr.returncode}"


def same(mode, a, b):
    ta, tb = a.split(), b.split()
    if mode == "exact": return ta == tb
    if mode == "float": return len(ta) == len(tb) and all(abs(float(x) - float(y)) <= 0.011 for x, y in zip(ta, tb))
    if mode == "head1": return ta[:1] == tb[:1]            # any witness; the length must agree
    if mode == "head2": return a.splitlines()[:2] == b.splitlines()[:2]   # feasibility + cost; plan may tie
    if mode == "len": return len(a.strip()) == len(b.strip())            # any LCS of the best length
    raise ValueError(mode)


def audit(spec, runs, tmp, keep):
    rel, orig, topic, limits, gen, mode = spec
    name = rel.split("/")[-1]
    r = {"port": rel, "topic": topic, "limits": limits, "compare": mode}
    d = os.path.join(tmp, name); os.makedirs(d)
    src = os.path.join(d, name + ".cp")
    with open(P(rel + ".cp")) as f, open(src, "w") as g: g.write(f.read())
    cpp = os.path.join(d, "port.cpp")
    t0 = time.perf_counter(); b = sh([CPL, "build", P(rel + ".cp"), "-o", cpp])   # in place, so its `cpl.toml` libraries resolve; r["cpl_build_ms"] = round((time.perf_counter() - t0) * 1e3, 1)
    if b.returncode: r["status"] = "cpl build failed: " + (b.stderr or b.stdout).strip().splitlines()[0]; return r
    r["emitted_bytes"] = os.path.getsize(cpp); r["size_ok"] = r["emitted_bytes"] < SIZE_LIMIT
    r["cp_lines"] = sum(1 for _ in open(src)); r["orig_lines"] = sum(1 for _ in open(orig))

    osrc = orig
    if name in bc5.DRIVERS:                       # grader-style originals need a stdin driver
        if name == "noiwc22p3":
            with open(os.path.join(d, "word.h"), "w") as f: f.write(bc5.WORD_H)
        osrc = os.path.join(d, "driver.cpp")
        with open(osrc, "w") as f: f.write(bc5.DRIVERS[name].replace("{orig}", orig))
    bins = {}
    for tag, s, flags in (("orig", osrc, ["-O2"]), ("port", cpp, ["-O2"]), ("asan", cpp, ["-O1", "-g", "-fsanitize=address,undefined", "-fno-sanitize-recover=undefined"])):
        bins[tag] = os.path.join(d, tag)
        cxx = ["g++", "-std=c++17"]
        if tag == "orig" and "/ecnerwala/" in s:           # upstream originals use C++20 (std::span)
            cxx = ["g++", "-std=c++20"]
        t0 = time.perf_counter(); c = sh([*cxx, *flags, "-I", d, s, "-o", bins[tag]])
        r[f"gxx_{tag}_s"] = round(time.perf_counter() - t0, 2)
        if c.returncode: r["status"] = f"g++ failed on {tag}: " + c.stderr.strip().splitlines()[0][:200]; return r

    def run_orig(path):
        timed(bins["orig"], path, path + ".gen"); return open(path + ".gen").read()
    ctx = {"run_orig": run_orig}

    med = os.path.join(d, "medium.in"); gen(med, False, ctx)
    _, _, st_o = timed(bins["orig"], med, med + ".o")
    _, _, st_a = timed(bins["asan"], med, med + ".a", pre=gb_stack, env={**os.environ, "ASAN_OPTIONS": "detect_leaks=0"})
    r["medium_match"] = st_o == "ok" and st_a == "ok" and same(mode, open(med + ".o").read(), open(med + ".a").read())
    r["sanitizer"] = "clean" if st_a == "ok" else st_a + ": " + " | ".join(open(med + ".a.err").read().strip().splitlines()[:2])[:300]

    big = os.path.join(d, "max.in"); gen(big, True, ctx)
    r["input_bytes"] = os.path.getsize(big)
    for tag in ("orig", "port"):
        ws, ms, sts = [], [], []
        for _ in range(runs):
            w, m, st = timed(bins[tag], big, big + "." + tag); ws.append(w); ms.append(m); sts.append(st)
            if st != "ok": break
        r[f"{tag}_s"] = round(statistics.median(ws), 4); r[f"{tag}_rss_kb"] = max(ms); r[f"{tag}_run"] = sts[-1]
    r["max_match"] = r["orig_run"] == "ok" and r["port_run"] == "ok" and same(mode, open(big + ".orig").read(), open(big + ".port").read())
    r["time_ratio"] = round(r["port_s"] / max(r["orig_s"], 1e-4), 2)
    r["rss_ratio"] = round(r["port_rss_kb"] / max(r["orig_rss_kb"], 1), 2)
    r["status"] = "ok" if r["max_match"] and r["medium_match"] and r["sanitizer"] == "clean" and r["size_ok"] else "FAIL"
    if keep: r["dir"] = d
    return r


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--ports", help="comma-separated port names (last path component is enough)")
    ap.add_argument("--runs", type=int, default=3)
    ap.add_argument("--json")
    ap.add_argument("--list", action="store_true")
    ap.add_argument("--ecnerwala", action="store_true", help="also run the ecnerwala ports from corpus_generators.ECNERWALA_SPECS")
    ap.add_argument("--keep", help="keep work files in this directory")
    a = ap.parse_args()
    import corpus_generators as cg
    specs = SPECS + (cg.ECNERWALA_SPECS if a.ecnerwala else [])
    if a.ports:
        want = set(a.ports.split(","))
        specs = [s for s in SPECS + cg.ECNERWALA_SPECS if s[0] in want or s[0].split("/")[-1] in want]
        missing = want - {s[0] for s in specs} - {s[0].split("/")[-1] for s in specs}
        if missing: sys.exit(f"--ports: no spec for {', '.join(sorted(missing))}")
    specs = [s for s in specs if s[4] is not None]
    if a.list:
        for s in specs: print(f"{s[0]:40s} {s[2]:20s} {s[3]}")
        return 0
    if not os.path.exists(CPL): sys.exit(f"{CPL} not found: cargo build --release -p cpl-cli")
    tmp = a.keep or tempfile.mkdtemp(prefix="cpl-maxn-")
    os.makedirs(tmp, exist_ok=True)
    out = []
    for s in specs:
        try:
            r = audit(s, a.runs, tmp, a.keep)
        except Exception as e:              # a broken generator must not stop the sweep
            r = {"port": s[0], "status": f"harness error: {e!r}"}
        out.append(r)
        if "time_ratio" in r:
            print(f"{r['status']:4s} {r['port']:38s} {r['orig_s']:7.3f}s -> {r['port_s']:7.3f}s x{r['time_ratio']:<6} "
                  f"rss {r['orig_rss_kb'] // 1024:5d} -> {r['port_rss_kb'] // 1024:5d} MB x{r['rss_ratio']:<5} "
                  f"{r['emitted_bytes']:6d} B  max={'same' if r['max_match'] else 'DIFF(' + r['port_run'] + ')'} "
                  f"med={'same' if r['medium_match'] else 'DIFF'} san={r['sanitizer'][:60]}", flush=True)
        else:
            print(f"FAIL {r['port']:38s} {r['status']}", flush=True)
    done = [r for r in out if "time_ratio" in r]
    if done:
        tr = sorted(r["time_ratio"] for r in done); mr = sorted(r["rss_ratio"] for r in done)
        print(f"\n{len(done)}/{len(out)} measured, {sum(r['status'] == 'ok' for r in out)} fully clean; "
              f"time ratio median {statistics.median(tr)} max {tr[-1]}; rss ratio median {statistics.median(mr)} max {mr[-1]}; "
              f"largest emitted file {max(r['emitted_bytes'] for r in done)} B")
    if a.json:
        with open(a.json, "w") as f: json.dump(out, f, indent=1)
    if not a.keep:
        import shutil; shutil.rmtree(tmp, ignore_errors=True)
    return 0 if all(r["status"] == "ok" for r in out) else 1


if __name__ == "__main__":
    sys.exit(main())
