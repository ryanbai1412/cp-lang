#!/usr/bin/env python3
"""Differential test for interactive ports: run the port and the original C++ against the same judge.

usage: interact.py port.cp orig.cpp judge.py [iters] [size]

judge.py defines `class Judge` with
    __init__(self, input_text, rng)   # the generated public input and a seeded random.Random
    first(self) -> str                # what the judge sends before any question
    reply(self, line) -> str | None   # answer to one line from the solution; None once it has finished
    ok(self) -> bool                  # verdict after the solution stops
"""
import importlib.util, os, random, subprocess, sys

def talk(exe, judge):
    p = subprocess.Popen([exe], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1)
    try:
        p.stdin.write(judge.first()); p.stdin.flush()
        for line in p.stdout:
            r = judge.reply(line.strip())
            if r is None:
                break
            p.stdin.write(r + "\n"); p.stdin.flush()
    except BrokenPipeError:
        pass
    finally:
        p.kill(); p.wait()
    return judge.ok()

def main():
    port, orig, judge_py = sys.argv[1:4]
    iters = int(sys.argv[4]) if len(sys.argv) > 4 else 100
    size = sys.argv[5] if len(sys.argv) > 5 else "6"
    here = os.path.dirname(os.path.abspath(__file__))
    base = port[:-3]
    r = subprocess.run([sys.executable, os.path.join(here, "cp.py"), "build", port], capture_output=True, text=True)
    if r.returncode:
        sys.exit(r.stdout + r.stderr)
    for out, flags in ((base + "_bin", []), (base + "_gen", ["-DCP_GEN_MODE"])):
        subprocess.run(["g++", "-std=c++17", "-O2", "-DLOCAL", *flags, "-o", out, base + ".cpp"], check=True)
    subprocess.run(["g++", "-std=c++17", "-O2", "-o", base + "_ref", orig], check=True)
    spec = importlib.util.spec_from_file_location("judge", judge_py)
    mod = importlib.util.module_from_spec(spec); spec.loader.exec_module(mod)
    for seed in range(1, iters + 1):
        inp = subprocess.run([base + "_gen", str(seed), size], capture_output=True, text=True).stdout
        for who, exe in (("ours", base + "_bin"), ("ref", base + "_ref")):
            if not talk(exe, mod.Judge(inp, random.Random(seed))):
                sys.exit(f"WRONG seed={seed} ({who})\ninput:\n{inp}")
    print(f"interact OK: {iters} conversations accepted for both the port and the original")

if __name__ == "__main__":
    main()
