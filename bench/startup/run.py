#!/usr/bin/env python3
"""Startup CPU of a tiny program: bench/startup/tiny.cp, built by `cpl build`,
against tiny.cpp (bits/stdc++, scanf/printf), both with g++ -std=c++17 -O2.

    python3 bench/startup/run.py [--cpl PATH] [--runs N] [--check]

Each run forks and execs one program, the two taking turns, and wait4 gives
its user + sys CPU time; the minimum of `--runs` runs is the time. --check
fails when the outputs differ or tiny.cp's time is over `RATIO` times tiny.cpp's.
"""
import argparse, hashlib, os, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))

# How many times tiny.cpp's CPU time tiny.cp's may take.
RATIO = 1.02

# cpu RUNS INPUT A B: the least user + sys microseconds of A and of B over
# RUNS runs each, in turns, so a slow stretch of the machine hits both.
LAUNCHER = r"""
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/resource.h>
#include <sys/wait.h>
#include <unistd.h>
static long cpu(const char *in, char *exe) {
    pid_t p = fork();
    if (p < 0) exit(125);
    if (p == 0) {
        int i = open(in, O_RDONLY), o = open("/dev/null", O_WRONLY);
        if (i < 0 || o < 0 || dup2(i, 0) < 0 || dup2(o, 1) < 0) _exit(126);
        char *argv[] = {exe, 0};
        execv(exe, argv);
        _exit(127);
    }
    int st;
    struct rusage u;
    if (wait4(p, &st, 0, &u) < 0 || !WIFEXITED(st) || WEXITSTATUS(st)) exit(1);
    return (u.ru_utime.tv_sec + u.ru_stime.tv_sec) * 1000000L + u.ru_utime.tv_usec + u.ru_stime.tv_usec;
}
int main(int argc, char **argv) {
    int runs = atoi(argv[1]);
    long best[2] = {-1, -1};
    for (int k = 0; k < runs; k++)
        for (int j = 0; j < 2; j++) {
            long t = cpu(argv[2], argv[3 + j]);
            if (best[j] < 0 || t < best[j]) best[j] = t;
        }
    printf("%ld %ld\n", best[0], best[1]);
}
"""


def launcher():
    """The timing launcher, built once with gcc and cached by source."""
    exe = os.path.join(ROOT, "target", "startup", "cpu-" + hashlib.sha256(LAUNCHER.encode()).hexdigest()[:16])
    if not os.path.exists(exe):
        os.makedirs(os.path.dirname(exe), exist_ok=True)
        src, tmp = f"{exe}.{os.getpid()}.c", f"{exe}.{os.getpid()}"
        with open(src, "w") as f:
            f.write(LAUNCHER)
        subprocess.run(["gcc", "-O2", "-o", tmp, src], check=True, capture_output=True)
        os.remove(src)
        os.replace(tmp, exe)
    return exe


def verdict(ours, hand, same):
    """Messages for tiny.cp's and tiny.cpp's microseconds and whether --check fails."""
    if not same:
        return ["startup: tiny.cp prints something else than tiny.cpp"], True
    if ours > hand * RATIO:
        return [f"startup: tiny.cp takes {ours} us of CPU, over {RATIO}x tiny.cpp's {hand} us"], True
    return [], False


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--cpl", default=os.environ.get("CPL", os.path.join(ROOT, "target", "release", "cpl")))
    ap.add_argument("--runs", type=int, default=200)
    ap.add_argument("--check", action="store_true")
    args = ap.parse_args()
    with tempfile.TemporaryDirectory() as tmp:
        inp = os.path.join(tmp, "in")
        with open(inp, "w") as f:
            f.write("7\n")
        ours_cpp = os.path.join(tmp, "tiny_cp.cpp")
        subprocess.run([args.cpl, "build", os.path.join(HERE, "tiny.cp"), "-o", ours_cpp],
                       check=True, capture_output=True)
        exes, outs = [], []
        for src, name in ((ours_cpp, "ours"), (os.path.join(HERE, "tiny.cpp"), "hand")):
            exe = os.path.join(tmp, name)
            subprocess.run(["g++", "-std=c++17", "-O2", "-w", "-o", exe, src], check=True)
            with open(inp) as i:
                outs.append(subprocess.run([exe], stdin=i, capture_output=True, check=True).stdout)
            exes.append(exe)
        got = subprocess.run([launcher(), str(args.runs), inp, *exes], check=True, capture_output=True, text=True)
        ours, hand = map(int, got.stdout.split())
    print("| program | CPU (us, min of %d) |" % args.runs)
    print("|---|---|")
    print(f"| tiny.cp | {ours} |")
    print(f"| tiny.cpp | {hand} |")
    messages, failed = verdict(ours, hand, outs[0] == outs[1])
    for m in messages:
        print(m, file=sys.stderr)
    return 1 if args.check and failed else 0


if __name__ == "__main__":
    sys.exit(main())
