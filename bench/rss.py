#!/usr/bin/env python3
"""Peak RSS of the emitted C++ against matched C++, shrink-only.

    python3 bench/rss.py PERF_JSON [--check]

PERF_JSON is `bench/v03_perf_bench.py --json-out`. The bars (decision 470,
performance/R5 and R6): idiomatic p95 <= 1.5x and max <= 2x, and the better
of idiomatic and tuned <= 1.1x. bench/rss-over.txt lists every program over
1.5x idiomatic (`name idiom RATIO`) or over 1.1x tuned (`name tuned RATIO`),
each with the ratio it may not grow past. --check fails when a program is
over its bar but not listed, grows more than NOISE past its listed ratio, or
is listed but now meets its bar (take it off, so the list only shrinks).
"""
import argparse, json, os, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OVER = os.path.join(ROOT, "bench", "rss-over.txt")
BARS = {"idiom": 1.5, "tuned": 1.1}
NOISE = 1.10


def listed(path=OVER):
    out = {}
    with open(path) as f:
        for line in f:
            words = line.split("#")[0].split()
            if words:
                out[(words[0], words[1])] = float(words[2])
    return out


def ratios(rows):
    """(name, kind) -> ratio for every row with a measured RSS."""
    out = {}
    for r in rows:
        x = r.get("rss_ratio")
        if not x:
            continue
        out[(r["name"], "idiom")] = x
        out[(r["name"], "tuned")] = min(x, r.get("tuned_rss_ratio") or x)
    return out


def verdict(measured, known):
    messages, failed = [], False
    for key, x in sorted(measured.items()):
        name, kind = key
        bar = BARS[kind]
        if x <= bar:
            if key in known:
                failed = True
                messages.append(f"rss: {name} {kind} is {x:.3f}x, within {bar}x now: take it off bench/rss-over.txt")
            continue
        if key not in known:
            failed = True
            messages.append(f"rss: {name} {kind} is {x:.3f}x, over {bar}x and not in bench/rss-over.txt")
        elif x > known[key] * NOISE:
            failed = True
            messages.append(f"rss: {name} {kind} grew to {x:.3f}x from the listed {known[key]:.3f}x")
    return messages, failed


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("perf_json")
    ap.add_argument("--check", action="store_true")
    args = ap.parse_args()
    with open(args.perf_json) as f:
        data = json.load(f)
    measured = ratios(data["rows"])
    messages, failed = verdict(measured, listed())
    for m in messages:
        print(m)
    idiom = sorted(x for (_, k), x in measured.items() if k == "idiom")
    tuned = sorted(x for (_, k), x in measured.items() if k == "tuned")
    if idiom:
        p95 = idiom[min(len(idiom) - 1, int(0.95 * len(idiom)))]
        print(f"rss: {len(idiom)} programs; idiomatic p95 {p95:.3f}x (bar 1.5), max {idiom[-1]:.3f}x (bar 2),"
              f" {sum(x > 1.5 for x in idiom)} over 1.5x; tuned max {tuned[-1]:.3f}x (bar 1.1),"
              f" {sum(x > 1.1 for x in tuned)} over")
    sys.exit(1 if args.check and failed else 0)


if __name__ == "__main__":
    main()
