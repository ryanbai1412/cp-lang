"""Three-way merge of scripts/debt-ratchet.json for a rebase conflict.

Every count is the lower of the two sides, so neither side's raise survives
the merge; a name or key either side dropped stays dropped. A name or key
only one side added is kept (a new counter or a decided edge), and the
raise check against main still judges it.

    uv run --quiet python scripts/ratchet_merge.py PATH   # in a conflicted rebase
    python3 scripts/ratchet_merge.py --driver %O %A %B    # git's merge driver (.gitattributes)
"""

import json
import subprocess
import sys

GONE = object()
# Both sides move these for a stated reason (one adds cases, the other passes some), so the
# merge keeps both moves; the raise check against main still judges the result.
ADDITIVE = {"next_not_passing"}


def merge(base, ours, theirs):
    if GONE in (ours, theirs):
        return ours if ours is not GONE and base is GONE else theirs if base is GONE else GONE
    if type(ours) is int and type(theirs) is int:
        # `ours` is main in a rebase (the driver's %A, land.sh's stage 2): a number main raised
        # landed through main's own check and stays, less what the lane paid off; 768cb11b6
        # took the lower and set old_pipeline_lines back under main's own pinned std.
        if type(base) is int and ours > base:
            return ours + min(0, theirs - base)
        return min(ours, theirs)
    if isinstance(ours, dict) and isinstance(theirs, dict):
        b = base if isinstance(base, dict) else {}
        out = {}
        for k in [*ours, *(k for k in theirs if k not in ours)]:
            if k in ADDITIVE and all(type(d.get(k)) is int for d in (b, ours, theirs)):
                out[k] = ours[k] + theirs[k] - b[k]
                continue
            v = merge(b.get(k, GONE), ours.get(k, GONE), theirs.get(k, GONE))
            if v is not GONE:
                out[k] = v
        return out
    if isinstance(ours, list) and isinstance(theirs, list):
        b = set(base) if isinstance(base, list) else set()
        o, t = set(ours), set(theirs)
        return sorted((o | (t - b)) - (b - t) - (b - o))
    return theirs if ours == base else ours


def stage(path, n):
    r = subprocess.run(["git", "show", f":{n}:{path}"], capture_output=True, text=True)
    return json.loads(r.stdout) if r.returncode == 0 else {}


def driver(base, ours, theirs):
    """git's merge driver: every merge and rebase, a lane's own included, merges the ratchet
    mechanically instead of leaving conflict markers in it."""
    def read(path):
        try:
            with open(path, encoding="utf-8") as f:
                return json.load(f)
        except (OSError, ValueError):
            return {}
    merged = merge(read(base), read(ours), read(theirs))
    with open(ours, "w", encoding="utf-8") as f:
        f.write(json.dumps(merged, indent=2) + "\n")
    return 0


def main(argv):
    if argv[1:2] == ["--driver"]:
        return driver(*argv[2:5])
    path = argv[1]
    # In a rebase, stage 2 is the upstream being rebased onto, stage 3 the commit replayed.
    merged = merge(stage(path, 1), stage(path, 2), stage(path, 3))
    with open(path, "w", encoding="utf-8") as f:
        f.write(json.dumps(merged, indent=2) + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
