#!/usr/bin/env python3
"""Stack frame of deep recursive fns: each case's port, built by `cpl build`,
compiled with g++ -std=c++17 -O2 -fstack-usage; its recursive fn's frame in
bytes against the bound recorded here.

    python3 bench/frames.py [--cpl PATH] [--check] [CASE...]

A recursive fn's frame is paid at every level of the recursion: at depth
1e5 each 8 bytes is 0.8 MB of peak RSS. --check fails when a frame is over
its bound. Bounds only shrink: lower one when a frame drops below it.
"""
import argparse, os, re, subprocess, sys, tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# case: (corpus package, the recursive fn's C++ name prefix, frame bound in bytes).
# `outer::` names the recursive closure declared inside fn `outer`.
CASES = {
    "apio07p1": ("apio07p1", "span", 112),
    "cf2187f2": ("cf2187f2", "ends_", 96),
    "year2018p7": ("year2018p7", "shared::", 192),
}


def frames(su_text):
    """{fn name: bytes} from a .su file, outside any lambda."""
    out = {}
    for line in su_text.splitlines():
        parts = line.split("\t")
        if len(parts) < 2 or "<lambda" in parts[0]:
            continue
        m = re.search(r"(?:prog::)?(\w+)\(", parts[0])
        if m:
            out[m.group(1)] = max(out.get(m.group(1), 0), int(parts[1]))
    return out


def closure_frames(su_text, outer):
    """The frames of the lambdas declared inside fn `outer`."""
    pat = re.compile(rf"(?:prog::)?{re.escape(outer)}\([^)]*\)::<lambda")
    return [int(parts[1]) for parts in (line.split("\t") for line in su_text.splitlines())
            if len(parts) >= 2 and pat.search(parts[0])]


def frame_of(su_text, prefix):
    """The largest frame of a fn whose name starts with `prefix` (or of a
    closure inside fn `outer`, for `outer::`), or None."""
    if prefix.endswith("::"):
        sizes = closure_frames(su_text, prefix[:-2])
    else:
        sizes = [b for name, b in frames(su_text).items() if name.startswith(prefix)]
    return max(sizes) if sizes else None


def verdict(rows):
    """Messages for `rows` (case, frame bytes or None, bound) and whether --check fails."""
    bad = []
    for case, got, bound in rows:
        if got is None:
            bad.append(f"frames: {case} has no recursive fn to measure")
        elif got > bound:
            bad.append(f"frames: {case}'s recursive frame is {got} B, over its bound of {bound} B")
    return bad, bool(bad)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("cases", nargs="*", default=list(CASES))
    ap.add_argument("--cpl", default=os.environ.get("CPL", os.path.join(ROOT, "target", "release", "cpl")))
    ap.add_argument("--check", action="store_true")
    args = ap.parse_args()
    rows = []
    print("| case | frame (B) | bound (B) |")
    print("|---|---|---|")
    with tempfile.TemporaryDirectory() as tmp:
        for case in args.cases:
            package, prefix, bound = CASES[case]
            cpp = os.path.join(tmp, case + ".cpp")
            port = os.path.join(ROOT, "corpus", "problems", package, "port.cp")
            subprocess.run([args.cpl, "build", port, "-o", cpp], check=True, capture_output=True)
            obj = os.path.join(tmp, case + ".o")
            subprocess.run(["g++", "-std=c++17", "-O2", "-fstack-usage", "-c", cpp, "-o", obj],
                           check=True, cwd=tmp)
            with open(os.path.join(tmp, case + ".su")) as f:
                got = frame_of(f.read(), prefix)
            rows.append((case, got, bound))
            print(f"| {case} | {got} | {bound} |", flush=True)
    messages, failed = verdict(rows)
    for m in messages:
        print(m, file=sys.stderr)
    return 1 if args.check and failed else 0


if __name__ == "__main__":
    sys.exit(main())
