"""Three-way merge of scripts/debt-ratchet.json for a rebase conflict.

Each side's tightening survives: a count either side lowered takes the lower
value, a name either side dropped stays dropped, and a name or key only one
side added is kept. Nothing is raised that neither side raised, so the merged
baseline is at most as loose as main's plus the commit's own change, which
`debt-ratchet.py --check` still judges.

    uv run --quiet python scripts/ratchet_merge.py PATH   # in a conflicted rebase
"""

import json
import subprocess
import sys

GONE = object()


def merge(base, ours, theirs):
    if ours == theirs:
        return ours
    if theirs == base:
        return ours
    if ours == base:
        return theirs
    if GONE in (ours, theirs):
        return GONE
    if isinstance(ours, bool) or isinstance(theirs, bool):
        return ours
    if isinstance(ours, int) and isinstance(theirs, int):
        return min(ours, theirs)
    if isinstance(ours, dict) and isinstance(theirs, dict):
        b = base if isinstance(base, dict) else {}
        out = {}
        for k in [*ours, *(k for k in theirs if k not in ours)]:
            v = merge(b.get(k, GONE), ours.get(k, GONE), theirs.get(k, GONE))
            if v is not GONE:
                out[k] = v
        return out
    if isinstance(ours, list) and isinstance(theirs, list):
        b = set(base) if isinstance(base, list) else set()
        o, t = set(ours), set(theirs)
        return sorted((o | (t - b)) - (b - t) - (b - o))
    return ours


def stage(path, n):
    r = subprocess.run(["git", "show", f":{n}:{path}"], capture_output=True, text=True)
    return json.loads(r.stdout) if r.returncode == 0 else {}


def main(argv):
    path = argv[1]
    # In a rebase, stage 2 is the upstream being rebased onto, stage 3 the commit replayed.
    merged = merge(stage(path, 1), stage(path, 2), stage(path, 3))
    with open(path, "w", encoding="utf-8") as f:
        f.write(json.dumps(merged, indent=2) + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
