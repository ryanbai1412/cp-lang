#!/usr/bin/env python3
"""review-check.py: every new port has a dream and a taste review.

    python3 scripts/review-check.py            report
    python3 scripts/review-check.py --check    also fail on the rules below
    python3 scripts/review-check.py --update   lower the ratchet to today's count

A port is new when its name is not in research/reviews/baseline.json, the
ports that existed when the review gate started. A new port fails --check
when it has no proto/ports/<group>/dream/NAME.cp or no
research/reviews/NAME.md. A review starts with a `Score: N/5` line, or
`Score: pending` until the reviewer scores it (plan/PORTING.md).

The ratchet: while more new ports score under 4 than baseline.json's
`new_under_4_max`, a change that adds another new port fails; reviews and
rewrites still land. The report gives the median score, the count under 4
and the dream-token gap: port tokens over dream tokens, median over ports
with a dream.
"""
import argparse, json, os, re, statistics, subprocess, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import scoreboard as sb

REVIEWS_DIR = os.path.join(sb.ROOT, "research", "reviews")
BASELINE = os.path.join(REVIEWS_DIR, "baseline.json")
SCORE_RE = re.compile(r"(?m)^Score:\s*(?:(\d)\s*/\s*5|(pending))\b")


def parse_score(text):
    """1 to 5, "pending", or None when the review has no score line."""
    m = SCORE_RE.search(text)
    if not m:
        return None
    return "pending" if m[2] else int(m[1])


def review_path(name):
    return os.path.join(REVIEWS_DIR, f"{name}.md")


def dream_path(port):
    return os.path.join(os.path.dirname(port["abs_path"]), "dream", port["name"] + ".cp")


def load_reviews(ports):
    """name -> (score, path relative to the root), for ports with a review file."""
    out = {}
    for p in ports:
        path = review_path(p["name"])
        if os.path.isfile(path):
            out[p["name"]] = (parse_score(sb._read_text(path)), os.path.relpath(path, sb.ROOT))
    return out


def dream_gap(port):
    path = dream_path(port)
    if not os.path.isfile(path):
        return None
    dream = len(sb.tokenize_cp(sb._read_text(path)))
    ours = len(sb.tokenize_cp(sb._read_text(port["abs_path"])))
    return ours / dream if dream else None


def added_port_names(names):
    """Names among `names` whose .cp is added by the staged or unpushed changes."""
    ranges = ["--cached"]
    if subprocess.run(["git", "rev-parse", "-q", "--verify", "origin/main"], cwd=sb.ROOT,
                      capture_output=True).returncode == 0:
        ranges.append("origin/main...HEAD")
    added = set()
    for r in ranges:
        out = subprocess.run(["git", "diff", r, "--name-only", "--diff-filter=A", "--", "proto/ports"],
                             cwd=sb.ROOT, capture_output=True, text=True).stdout
        for line in out.splitlines():
            m = re.fullmatch(r"proto/ports/[^/]+/([^/]+)\.cp", line)
            if m and m[1] in names:
                added.add(m[1])
    return added


def evaluate(ports, baseline, reviews, dreams, added):
    """Pure core: (errors, stats). `dreams` is the set of names with a dream file,
    `added` the new names the change being checked adds."""
    old = set(baseline["ports"])
    new = [p["name"] for p in ports if p["name"] not in old]
    errors = []
    for name in new:
        if name not in dreams:
            errors.append(f"{name}: new port without a dream (proto/ports/<group>/dream/{name}.cp)")
        if name not in reviews:
            errors.append(f"{name}: new port without a review (research/reviews/{name}.md, "
                          "`Score: pending` until reviewed)")
        elif reviews[name] is None:
            errors.append(f"{name}: research/reviews/{name}.md has no `Score: N/5` or `Score: pending` line")
    scored = {n: s for n, s in reviews.items() if isinstance(s, int)}
    new_under = sorted(n for n in new if scored.get(n, 5) < 4)
    cap = baseline.get("new_under_4_max", 0)
    if len(new_under) > cap and added:
        errors.append(f"ratchet: {len(new_under)} new ports score under 4 ({', '.join(new_under)}), "
                      f"over the {cap} allowed, so no new port may land ({', '.join(sorted(added))}); "
                      "rewrite those first")
    stats = {
        "ports": len(ports),
        "new": len(new),
        "reviewed": len(scored),
        "pending": sum(1 for s in reviews.values() if s == "pending"),
        "median": statistics.median(scored.values()) if scored else None,
        "under_4": sum(1 for s in scored.values() if s < 4),
        "new_under_4": len(new_under),
        "cap": cap,
    }
    return errors, stats


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--check", action="store_true", help="exit 1 when a rule fails")
    ap.add_argument("--update", action="store_true", help="lower new_under_4_max to the current count")
    ap.add_argument("--quiet", action="store_true", help="print only the summary line and errors")
    args = ap.parse_args()

    ports = sb.discover_ports()
    with open(BASELINE) as f:
        baseline = json.load(f)
    reviews = {n: s for n, (s, _) in load_reviews(ports).items()}
    dreams = {p["name"] for p in ports if os.path.isfile(dream_path(p))}
    new = {p["name"] for p in ports} - set(baseline["ports"])
    errors, st = evaluate(ports, baseline, reviews, dreams, added_port_names(new) if args.check else set())
    gaps = [g for g in (dream_gap(p) for p in ports) if g]
    gap = f"{statistics.median(gaps):.2f}x over {len(gaps)}" if gaps else "none"
    median = f"{st['median']:g}" if st["median"] is not None else "none"
    print(f"reviews: {st['reviewed']}/{st['ports']} scored, median {median}, {st['under_4']} under 4, "
          f"{st['pending']} pending; new ports {st['new']}, {st['new_under_4']} under 4 "
          f"(ratchet {st['cap']}); dream-token gap {gap}")
    if not args.quiet:
        for p in ports:
            s = reviews.get(p["name"])
            if isinstance(s, int) and s < 4:
                print(f"  {s}/5 {p['name']}")
    if args.update and st["new_under_4"] < st["cap"]:
        baseline["new_under_4_max"] = st["new_under_4"]
        with open(BASELINE, "w") as f:
            json.dump(baseline, f, indent=1)
            f.write("\n")
        print(f"ratchet lowered to {st['new_under_4']}")
    for e in errors:
        print(f"review-check: {e}", file=sys.stderr)
    if args.check and errors:
        sys.exit(1)


if __name__ == "__main__":
    main()
