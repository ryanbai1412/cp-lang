#!/usr/bin/env python3
"""review-check.py: every new port has a dream and a taste review.

    python3 scripts/review-check.py            report
    python3 scripts/review-check.py --check    also fail on the rules below
    python3 scripts/review-check.py --update   lower the ratchet to today's count

A port is new when its name is not in research/reviews/baseline.json, the
ports that existed when the review gate started. A new port fails --check
when it has no corpus/problems/<id>/dream.cp or no
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
DREAM_RE = re.compile(r"(?m)^Dream:\s*(\d)\s*/\s*5\b")


def parse_score(text):
    """1 to 5, "pending", or None when the review has no score line."""
    score, is_pending = sb.parse_review_score_text(text)
    if is_pending:
        return "pending"
    return int(round(score)) if score is not None else None


def parse_dream_score(text):
    """The dream's own 1 to 5 score; under 4 means the dream is sent back."""
    m = DREAM_RE.search(text)
    return int(m[1]) if m else None


def review_path(name):
    return os.path.join(REVIEWS_DIR, f"{name}.md")


def dream_path(port):
    return os.path.join(os.path.dirname(port["abs_path"]), "dream.cp")


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


def touched_port_names(names):
    """Return (added, touched) port names among `names` changed in the working tree / index."""
    added = set()
    touched = set()
    for cmd in (
        ["git", "diff", "--cached", "--name-status", "--diff-filter=AM", "--", "corpus/problems"],
        ["git", "diff", "--name-status", "--diff-filter=AM", "--", "corpus/problems"],
    ):
        out = subprocess.run(cmd, cwd=sb.ROOT, capture_output=True, text=True).stdout
        for line in out.splitlines():
            parts = line.split("\t", 1)
            if len(parts) != 2:
                continue
            status, path = parts[0].strip(), parts[1].strip()
            m = re.fullmatch(r"corpus/problems/([^/]+)/port\.cp", path)
            if m and m[1] in names:
                touched.add(m[1])
                if status.startswith("A"):
                    added.add(m[1])
    return added, touched


def added_port_names(names):
    """Names among `names` whose .cp the staged change adds."""
    added, _ = touched_port_names(names)
    return added


def evaluate(ports, baseline, reviews, dreams, added, touched=None):
    """Pure core: (errors, stats).

    Attributable rules:
    - Every new port (`added`) needs a dream file and a review file (`Score: N/5` or `Score: pending`).
    - Every changed port (`touched`) needs a review file (`Score: N/5` or `Score: pending`).
    - Scores under 4 are tracked and reported in stats, never a commit blocker.
    """
    old = set(baseline["ports"])
    new = [p["name"] for p in ports if p["name"] not in old]
    errors = []
    check_reviews = set(added) | (set(touched) if touched is not None else set())
    for name in sorted(added):
        if name not in dreams:
            errors.append(f"{name}: new port without a dream (corpus/problems/{name}/dream.cp)")
    for name in sorted(check_reviews):
        if name not in reviews:
            kind = "new" if name in added else "changed"
            errors.append(
                f"{name}: {kind} port without a review (research/reviews/{name}.md, "
                "`Score: pending` until reviewed)"
            )
        elif reviews[name] is None:
            errors.append(f"{name}: research/reviews/{name}.md has no `Score: N/5` or `Score: pending` line")
    scored = {n: s for n, s in reviews.items() if isinstance(s, int)}
    new_under = sorted(n for n in new if scored.get(n, 5) < 4)
    cap = baseline.get("new_under_4_max", 0)
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
    dream_scores = {p["name"]: parse_dream_score(sb._read_text(review_path(p["name"])))
                    for p in ports if os.path.isfile(review_path(p["name"]))}
    dream_scores = {n: s for n, s in dream_scores.items() if s is not None}
    sent_back = sorted(n for n, s in dream_scores.items() if s < 4)
    all_names = {p["name"] for p in ports}
    added, touched = touched_port_names(all_names) if args.check else (set(), set())
    errors, st = evaluate(ports, baseline, reviews, dreams, added, touched=touched)
    gaps = [g for g in (dream_gap(p) for p in ports) if g]
    gap = f"{statistics.median(gaps):.2f}x over {len(gaps)}" if gaps else "none"
    median = f"{st['median']:g}" if st["median"] is not None else "none"
    print(f"reviews: {st['reviewed']}/{st['ports']} scored, median {median}, {st['under_4']} under 4, "
          f"{st['pending']} pending; new ports {st['new']}, {st['new_under_4']} under 4 "
          f"(tracked, non-blocking); dream-token gap {gap}")
    dmed = f"{statistics.median(dream_scores.values()):g}" if dream_scores else "none"
    print(f"dreams: {len(dream_scores)} scored, median {dmed}, {len(sent_back)} sent back"
          + (f" ({', '.join(sent_back)})" if sent_back else ""))
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
