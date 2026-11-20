#!/usr/bin/env python3
"""Microbenchmarks: each X.cp vs hand-written X_hand.cpp. Prints a Markdown table."""
import os, subprocess, sys, time
os.chdir(os.path.dirname(os.path.abspath(__file__)))
N = {"map": 10**7, "opt": 10**7, "set": 10**7, "gen": 10**7, "memo": 3000}
def run(exe, inp):
    t = time.time()
    p = subprocess.run(["/usr/bin/time", "-f", "%M", "./" + exe], input=inp, capture_output=True, text=True)
    return time.time() - t, int(p.stderr.strip().split()[-1]) / 1024, p.stdout.strip()
print("| bench | ours (s) | hand C++ (s) | ours mem (MB) | hand mem (MB) | same |")
print("|---|---|---|---|---|---|")
for b in sys.argv[1:] or N:
    r = subprocess.run([sys.executable, "../cp.py", "build", b + ".cp"], capture_output=True, text=True)
    if r.returncode:
        print(f"| {b} | build failed: {r.stderr.strip()[:80]} |"); continue
    for src, exe in ((b + ".cpp", b + "_ours"), (b + "_hand.cpp", b + "_hand")):
        subprocess.run(["g++", "-std=c++17", "-O2", "-o", exe, src], check=True, capture_output=True)
    inp = f"{N[b]}\n"
    ta, ma, oa = run(b + "_ours", inp)
    tb, mb, ob = run(b + "_hand", inp)
    print(f"| {b} | {ta:.2f} | {tb:.2f} | {ma:.0f} | {mb:.0f} | {'yes' if oa == ob else f'NO ({oa} vs {ob})'} |")
