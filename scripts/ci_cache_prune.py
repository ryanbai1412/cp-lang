"""Delete main's superseded Actions caches, so the live ones stay under GitHub's 10 GB.

A key saved per run or per hour (`...-<run id>`, `...-<YYYYMMDDHH>`, or a new
lockfile/recipe hash) leaves its older entries behind; restores take the
newest match, so the older ones only push the repository over its limit, and
GitHub then evicts the least recently used caches, including live g++ ones.
Every main cache whose key, less those suffixes, matches a newer main cache's
is deleted.

Usage: uv run python scripts/ci_cache_prune.py [--dry-run]
"""
import argparse
import os
import re

from ci_unstick import gh

SUPERSEDED = re.compile(r"-(?:[0-9a-f]{64}|\d{10,})(?=-|$)")


def family(key):
    return SUPERSEDED.sub("", key)


def stale(caches):
    """The main caches a newer main cache of the same family supersedes."""
    newest = {}
    for c in caches:
        if c["ref"] != "refs/heads/main" or family(c["key"]) == c["key"]:
            continue
        f = family(c["key"])
        if f not in newest or c["created_at"] > newest[f]["created_at"]:
            newest[f] = c
    return [c for c in caches if c["ref"] == "refs/heads/main" and family(c["key"]) in newest
            and family(c["key"]) != c["key"] and c is not newest[family(c["key"])]]


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--dry-run", action="store_true")
    a = ap.parse_args()
    repo = os.environ["GITHUB_REPOSITORY"]
    caches, page = [], 1
    while True:
        got = gh(f"repos/{repo}/actions/caches?per_page=100&page={page}")["actions_caches"]
        caches += got
        if len(got) < 100:
            break
        page += 1
    gone = stale(caches)
    for c in gone:
        if not a.dry_run:
            gh("-X", "DELETE", f"repos/{repo}/actions/caches/{c['id']}")
    mb = sum(c["size_in_bytes"] for c in gone) // 10**6
    print(f"ci-cache-prune: deleted {len(gone)} superseded cache(s), {mb} MB")


if __name__ == "__main__":
    main()
