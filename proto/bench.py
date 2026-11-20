#!/usr/bin/env python3
"""Per port: generated C++ size, and runtime/memory of ours vs original on large random inputs."""
import os, random, resource, subprocess, sys, time
ports = sys.argv[1:] or sorted(f[:-3] for f in os.listdir("ports") if f.endswith(".cp"))

def tree_input(name, rng):
    if name == "apio07p1":
        n = 100000
        kids = [[-1, -1] for _ in range(n + 1)]
        free = [(1, 0), (1, 1)]
        for v in range(2, n + 1):
            i = rng.randrange(len(free)); u, s = free[i]; free[i] = free[-1]; free.pop()
            kids[u][s] = v; free += [(v, 0), (v, 1)]
        return f"{n}\n" + "".join(f"{a} {b}\n" for a, b in kids[1:])
    if name == "ioi05p6":
        n, k = 100, 50
        return f"{n} {k}\n" + "".join(f"{rng.randint(0, 10000)} {rng.randrange(0, v)} {rng.randint(1, 10000)}\n"
                                       for v in range(1, n + 1))

    if name == "noi02p1":
        q = 500000
        return f"{q}\n" + "".join(f"{rng.choice('MC')} {rng.randint(1, 30000)} {rng.randint(1, 30000)}\n" for _ in range(q))
    if name == "noip09p2":
        import math
        t = 2000
        rows = []
        for _ in range(t):
            x, a0 = rng.randint(1, 2 * 10**9), rng.randint(1, 2 * 10**9)
            rows.append(f"{a0} {math.gcd(x, a0)} {x} {x}\n")
        return f"{t}\n" + "".join(rows)

SLOW = []

def run(exe, inp):
    t = time.time()
    p = subprocess.Popen(["/usr/bin/time", "-f", "%M", exe], stdin=subprocess.PIPE,
                         stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    out, err = p.communicate(inp, timeout=60)
    return time.time() - t, int(err.strip().split()[-1]), out

print("| port | generated C++ | ours (s) | original (s) | ours mem (MB) | original mem (MB) | same output |")
print("|---|---|---|---|---|---|---|")
for name in ports:
    base = f"ports/{name}"
    subprocess.run([sys.executable, "cp.py", "build", base + ".cp"], check=True, capture_output=True)
    for out, src, fl in ((base + "_bin", base + ".cpp", []), (base + "_gen", base + ".cpp", ["-DCP_GEN_MODE"]),
                         (base + "_ref", base + "_orig.cpp", ["-DNDEBUG"])):
        subprocess.run(["g++", "-std=c++17", "-O2", *fl, "-o", out, src], check=True, capture_output=True)
    rng = random.Random(1)
    worst = [0, 0, 0, 0]; same = True
    for seed in range(1, 4):
        inp = tree_input(name, rng)
        if inp is None:
            g = subprocess.run([base + "_gen", str(seed), "-1"], capture_output=True, text=True)
            inp = g.stdout
        ta, ma, oa = run(base + "_bin", inp)
        tb, mb, ob = run(base + "_ref", inp)
        same &= oa.split() == ob.split()
        worst = [max(worst[0], ta), max(worst[1], tb), max(worst[2], ma), max(worst[3], mb)]
    size = os.path.getsize(base + ".cpp")
    print(f"| {name} | {size / 1024:.1f} KB | {worst[0]:.2f} | {worst[1]:.2f} | {worst[2] / 1024:.0f} | {worst[3] / 1024:.0f} | {'yes' if same else 'NO'} |")
    if worst[0] > 1.2 * worst[1] + 0.02:
        SLOW.append(name)
        print(f"SLOW: {name} takes {worst[0]:.2f}s vs original {worst[1]:.2f}s (> 1.2x)", file=sys.stderr)
if SLOW:
    sys.exit(1)
